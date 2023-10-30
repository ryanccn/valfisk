use crate::Context;

pub mod ping;
pub mod pomelo;
pub mod presence;
pub mod say;
pub mod self_timeout;
pub mod shiggy;
pub mod translate;

pub fn to_vec() -> Vec<
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
