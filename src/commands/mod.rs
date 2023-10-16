use crate::Context;

mod ping;
mod pomelo;
mod presence;
mod say;
mod shiggy;
mod translate;

pub fn vec() -> Vec<
    ::poise::Command<
        <Context<'static> as poise::_GetGenerics>::U,
        <Context<'static> as poise::_GetGenerics>::E,
    >,
> {
    vec![
        ping::ping(),
        pomelo::pomelo(),
        presence::presence(),
        say::say(),
        shiggy::shiggy(),
        translate::translate(),
    ]
}
