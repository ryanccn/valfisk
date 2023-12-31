use color_eyre::eyre::{eyre, Result};
use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::Context;

use std::{sync::Once, time::Duration};
use tokio::time::timeout;

static V8_PLATFORM_INIT: Once = Once::new();

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum RunCodeResult {
    ReturnValue(String),
    NoReturnValue,
    Exception(String),
}

impl RunCodeResult {
    fn to_embed(&self) -> CreateEmbed {
        match self {
            RunCodeResult::ReturnValue(v) => CreateEmbed::default()
                .title("Success!")
                .description("```".to_owned() + v + "```")
                .color(0x4ade80),
            RunCodeResult::Exception(v) => CreateEmbed::default()
                .title("An exception occurred!")
                .description("```".to_owned() + v + "```")
                .color(0xef4444),
            RunCodeResult::NoReturnValue => CreateEmbed::default()
                .title("Missing return value!")
                .description("Did you forget to call `Valfisk.setReturnValue()`?")
                .color(0xfacc15),
        }
    }
}

fn build_valfisk_scope(
    context: v8::Local<'_, v8::Context>,
    scope: &mut v8::ContextScope<'_, v8::HandleScope<'_>>,
) {
    let valfisk_object = v8::Object::new(scope);

    {
        let k = v8::String::new_external_onebyte_static(scope, b"setReturnValue")
            .unwrap()
            .into();
        let v = v8::FunctionBuilder::<v8::Function>::new(
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut _rv: v8::ReturnValue| {
                let arg = args.get(0).to_rust_string_lossy(scope);
                scope.set_slot(arg);
            },
        )
        .build(scope)
        .unwrap()
        .into();

        valfisk_object.set(scope, k, v);
    }

    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        let k = v8::String::new_external_onebyte_static(scope, b"version")
            .unwrap()
            .into();
        let v = v8::String::new_external_onebyte_static(scope, version.as_bytes())
            .unwrap()
            .into();

        valfisk_object.set(scope, k, v);
    }

    let valfisk_global_key = v8::String::new_external_onebyte_static(scope, b"Valfisk")
        .unwrap()
        .into();

    context
        .global(scope)
        .set(scope, valfisk_global_key, valfisk_object.into());
}

fn build_script_origin<'a>(
    scope: &'a mut v8::ContextScope<'_, v8::HandleScope<'_>>,
) -> Result<v8::ScriptOrigin<'a>> {
    let resource_name = v8::String::new(scope, "valfisk.js")
        .ok_or_else(|| eyre!("Could not construct script origin in V8"))?
        .into();

    let source_map_url = v8::undefined(scope).into();

    Ok(v8::ScriptOrigin::new(
        scope,
        resource_name,
        0,
        0,
        false,
        0,
        source_map_url,
        false,
        false,
        true,
    ))
}

async fn run_code(code: &str) -> Result<RunCodeResult> {
    V8_PLATFORM_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();

        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });

    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());

    let scope = &mut v8::HandleScope::new(isolate);
    // scope.set_capture_stack_trace_for_uncaught_exceptions(true, 5);

    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    build_valfisk_scope(context, scope);

    let code = v8::script_compiler::Source::new(
        v8::String::new(scope, code).unwrap(),
        Some(&build_script_origin(scope)?),
    );

    let module = v8::script_compiler::compile_module(scope, code)
        .ok_or_else(|| eyre!("Could not compile code in V8"))?;

    if module
        .instantiate_module(scope, |_, _, _, _| None)
        .is_some()
    {
        let result_value = module
            .evaluate(scope)
            .ok_or_else(|| eyre!("Could not evaluate module in V8"))?;

        let result_promise = v8::Local::<v8::Promise>::try_from(result_value)?;

        loop {
            match result_promise.state() {
                v8::PromiseState::Fulfilled => {
                    let ret_value = scope.get_slot::<String>().cloned();

                    return Ok(match ret_value {
                        Some(rv) => RunCodeResult::ReturnValue(rv),
                        None => RunCodeResult::NoReturnValue,
                    });
                }
                v8::PromiseState::Rejected => {
                    return Ok(RunCodeResult::Exception(
                        result_promise.result(scope).to_rust_string_lossy(scope),
                    ))
                }
                v8::PromiseState::Pending => {}
            };
        }
    } else {
        return Ok(RunCodeResult::Exception(
            module.get_exception().to_rust_string_lossy(scope),
        ));
    }
}

/// Run JavaScript natively on the Valfisk runtime™
#[poise::command(slash_command, guild_only)]
pub async fn v8(
    ctx: Context<'_>,
    #[description = "The JavaScript code to execute"] code: String,
) -> Result<()> {
    ctx.defer().await?;

    match timeout(
        Duration::from_secs(10),
        tokio::spawn(async move {
            let code = code.clone();
            run_code(&code).await
        }),
    )
    .await
    {
        Ok(result) => {
            ctx.send(CreateReply::default().embed(result??.to_embed()))
                .await?;
        }
        Err(_) => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title("Timed out!")
                        .description("The script didn't exit after 10 seconds. What were you trying to do 🧐")
                        .color(0xef4444),
                ),
            )
            .await?;
        }
    }

    Ok(())
}
