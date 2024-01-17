use crate::Context;

mod lighthouse;
mod owo;
mod ping;
mod pomelo;
mod presence;
mod say;
mod self_timeout;
mod shiggy;
mod sysinfo;
mod translate;
mod version;

pub use presence::restore as restore_presence;

pub fn to_vec() -> Vec<
    poise::Command<
        <Context<'static> as poise::_GetGenerics>::U,
        <Context<'static> as poise::_GetGenerics>::E,
    >,
> {
    vec![
        lighthouse::lighthouse(),
        owo::owo(),
        ping::ping(),
        pomelo::pomelo(),
        presence::presence(),
        say::say(),
        self_timeout::self_timeout(),
        self_timeout::transparency(),
        shiggy::shiggy(),
        sysinfo::sysinfo(),
        translate::translate(),
        version::version(),
    ]
}
