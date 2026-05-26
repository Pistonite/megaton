use cu::pre::*;


pub fn install(keep: bool, clean: bool) -> cu::Result<()> {
    let home = crate::home::get_megaton_home()?;
    cu::check!(crate::cxxbridge::install(&home), "failed to install cxxbridge")?;
    cu::check!(crate::rust_toolchain::install(&home, keep, clean), "failed to install rust toolchain")?;
    Ok(())
}

pub fn check() -> cu::Result<()> {
    let home = crate::home::get_megaton_home()?;

    cu::info!("checking cxxbridge installation...");
    let mut needs_install = false;
    let mut has_error = false;
    match crate::cxxbridge::check(&home, true) {
        Err(e) => {
            cu::error!("{e:?}");
            needs_install = true;
            has_error = true;
        }
        Ok(None) => {
            cu::warn!("cxxbridge not found");
            needs_install = true;
        }
        Ok(Some(info)) => {
            if info.version != crate::cxxbridge::BLESSED_VERSION {
                cu::warn!("blessed version is newer: {}", crate::cxxbridge::BLESSED_VERSION);
                needs_install = true;
            }
        }
    }

    cu::info!("checking megaton rust installation...");
    match crate::rust_toolchain::check(true) {
        Err(e) => {
            cu::error!("{e:?}");
            needs_install = true;
            has_error = true;
        }
        Ok(None) => {
            cu::warn!("megaton rust toolchain not found");
            needs_install = true;
        }
        Ok(Some(info)) => {
            if info.commit_hash.is_none_or(|x| x != crate::rust_toolchain::BLESSED_COMMIT) {
                cu::warn!("blessed version is newer: {} ({})",
                    crate::rust_toolchain::BLESSED_VERSION,
                    crate::rust_toolchain::BLESSED_COMMIT,
                );
                needs_install = true;
            }
        }
    }

    if needs_install {
        cu::hint!("run `megaton toolchain install` to install/update the toolchain");
    }

    if has_error {
        cu::bail!("there were error(s) checking the toolchain status");
    }

    Ok(())
}

pub fn remove() -> cu::Result<()> {
    let home = crate::home::get_megaton_home()?;
    cu::check!(crate::cxxbridge::remove(&home), "failed to uninstall cxxbridge")?;
    cu::info!("uninstalled cxxbridge");

    if let Ok(None) = crate::rust_toolchain::check(false) {
        cu::info!("megaton rust toolchain is not installed.");
    } else {
        cu::check!(crate::rust_toolchain::remove(&home), "failed to uninstall megaton rust toolchain")?;
        cu::info!("uninstalledk megaton rust toolchain");
    }

    Ok(())
}

pub fn clean() -> cu::Result<()> {
    let home = crate::home::get_megaton_home()?;
    cu::check!(crate::rust_toolchain::clean(&home), "failed to clean megaton rust toolchain")?;
    Ok(())
}
