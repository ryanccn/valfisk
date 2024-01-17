use poise::serenity_prelude::User;

pub fn unique_username(user: &User) -> String {
    let mut ret = user.name.clone();
    if let Some(discrim) = user.discriminator {
        ret.push_str(&format!("#{discrim}"));
    }

    ret
}
