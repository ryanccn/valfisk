use crate::Context;

mod ping;
mod pomelo;
mod presence;
mod say;
mod self_timeout;
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
        self_timeout::self_timeout(),
        shiggy::shiggy(),
        translate::translate(),
    ]
}
