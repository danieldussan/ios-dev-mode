//! All user-facing strings, in es/en/pt. Templates use `{v}`, `{name}`, `{model}`, `{detail}`
//! placeholders filled by the caller with `str::replace`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
    Pt,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Es, Lang::En, Lang::Pt];

    pub fn name(self) -> &'static str {
        match self {
            Lang::Es => "Español",
            Lang::En => "English",
            Lang::Pt => "Português",
        }
    }

    pub fn from_locale(s: &str) -> Lang {
        let s = s.to_ascii_lowercase();
        if s.starts_with("en") {
            Lang::En
        } else if s.starts_with("pt") {
            Lang::Pt
        } else {
            Lang::Es
        }
    }

    pub fn from_env() -> Lang {
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty())
            .map_or(Lang::Es, |v| Lang::from_locale(&v))
    }
}

/// One row per key: the compiler enforces exactly three translations (es, en, pt).
macro_rules! strings {
    ($($key:ident => [$es:expr, $en:expr, $pt:expr],)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Key { $($key),* }

        impl Key {
            #[cfg(test)]
            pub const ALL: &'static [Key] = &[$(Key::$key),*];
        }

        pub fn t(lang: Lang, key: Key) -> &'static str {
            let [es, en, pt] = match key { $(Key::$key => [$es, $en, $pt]),* };
            match lang {
                Lang::Es => es,
                Lang::En => en,
                Lang::Pt => pt,
            }
        }
    };
}

strings! {
    WindowTitle => ["Modo desarrollador iOS", "iOS Developer Mode", "Modo de Desenvolvedor iOS"],
    MadeBy => ["Hecho por danieldussan", "Made by danieldussan", "Feito por danieldussan"],
    Language => ["Idioma", "Language", "Idioma"],

    BtnVerify => ["Verificar", "Check", "Verificar"],
    BtnRetry => ["Reintentar", "Retry", "Tentar novamente"],
    BtnActivate => ["Activar", "Activate", "Ativar"],
    BtnWorking => ["Trabajando…", "Working…", "Trabalhando…"],
    BtnVerifyNow => [
        "Ya reinicié / Verificar ahora",
        "I restarted / Check now",
        "Já reiniciei / Verificar agora"
    ],
    Cancel => ["Cancelar", "Cancel", "Cancelar"],
    Details => ["Detalles", "Details", "Detalhes"],

    StepDetect => ["Detectar iPhone", "Detect iPhone", "Detectar iPhone"],
    StepReveal => [
        "Mostrar opción en Ajustes",
        "Show option in Settings",
        "Mostrar opção em Ajustes"
    ],
    StepEnable => [
        "Activar el modo en Ajustes",
        "Turn it on in Settings",
        "Ativar o modo em Ajustes"
    ],
    StepReboot => [
        "Reiniciar cuando el iPhone lo pida",
        "Restart when the iPhone asks",
        "Reiniciar quando o iPhone pedir"
    ],
    StepReconnect => ["Reconexión", "Reconnection", "Reconexão"],
    StepAccept => ["Enviar confirmación", "Send confirmation", "Enviar confirmação"],
    StepConfirm => ["Confirmar en el iPhone", "Confirm on the iPhone", "Confirmar no iPhone"],

    TitleIdle => ["Listo para verificar", "Ready to check", "Pronto para verificar"],
    BodyIdle => [
        "Conecta tu iPhone por USB y toca Verificar.",
        "Connect your iPhone via USB and tap Check.",
        "Conecte seu iPhone por USB e toque em Verificar."
    ],
    TitleNoUsbmuxd => [
        "Sin conexión con el iPhone",
        "No connection to the iPhone",
        "Sem conexão com o iPhone"
    ],
    BodyNoUsbmuxd => [
        "Conecta el cable y desbloquea el iPhone con tu código (tras un reinicio no responde hasta entonces). Si sigue igual, instala iTunes o Apple Devices desde la web de Apple. Se reintenta solo.",
        "Plug in the cable and unlock the iPhone with your passcode (after a restart it does not respond until then). If nothing changes, install iTunes or Apple Devices from Apple's website. It retries on its own.",
        "Conecte o cabo e desbloqueie o iPhone com seu código (após reiniciar, ele não responde até então). Se continuar igual, instale o iTunes ou o Apple Devices pelo site da Apple. Tenta novamente sozinho."
    ],
    PopupDesc => [
        "Conecta el cable y desbloquea el iPhone con tu código (tras un reinicio no responde hasta entonces). Si sigue igual, instala iTunes o Apple Devices desde la web de Apple. La app reintenta sola.",
        "Plug in the cable and unlock the iPhone with your passcode (after a restart it does not respond until then). If nothing changes, install iTunes or Apple Devices from Apple's website. The app retries on its own.",
        "Conecte o cabo e desbloqueie o iPhone com seu código (após reiniciar, ele não responde até então). Se continuar igual, instale o iTunes ou o Apple Devices pelo site da Apple. O app tenta novamente sozinho."
    ],
    PopupOpen => ["Abrir página de Apple", "Open Apple's page", "Abrir página da Apple"],
    PopupClose => ["Cerrar", "Close", "Fechar"],
    TitleNoDevice => ["No hay iPhone", "No iPhone found", "Nenhum iPhone"],
    BodyNoDevice => [
        "Conecta tu iPhone por USB, desbloquéalo y toca Confiar",
        "Connect your iPhone via USB, unlock it and tap Trust",
        "Conecte seu iPhone por USB, desbloqueie-o e toque em Confiar"
    ],
    TitleNotTrusted => ["iPhone sin confianza", "iPhone not trusted", "iPhone não confiável"],
    BodyNotTrusted => [
        "Toca Confiar en el iPhone y vuelve a intentar",
        "Tap Trust on the iPhone and try again",
        "Toque em Confiar no iPhone e tente novamente"
    ],
    TitleNotNeeded => ["No es necesario", "Not needed", "Não é necessário"],
    BodyNotNeeded => [
        "Este iPhone (iOS {v}) no necesita modo desarrollador",
        "This iPhone (iOS {v}) does not need Developer Mode",
        "Este iPhone (iOS {v}) não precisa do modo de desenvolvedor"
    ],
    TitleReady => ["iPhone detectado", "iPhone detected", "iPhone detectado"],
    BodyReady => [
        "{name} · {model} · iOS {v}\nEl modo desarrollador está desactivado.",
        "{name} · {model} · iOS {v}\nDeveloper Mode is off.",
        "{name} · {model} · iOS {v}\nO modo de desenvolvedor está desativado."
    ],
    TitleWorking => ["Trabajando…", "Working…", "Trabalhando…"],
    BodyUnlockPhone => ["Desbloquea tu iPhone", "Unlock your iPhone", "Desbloqueie seu iPhone"],
    BodyDontUnplug => [
        "No desconectes el cable.",
        "Do not unplug the cable.",
        "Não desconecte o cabo."
    ],
    TitleSettings => [
        "Actívalo en Ajustes",
        "Turn it on in Settings",
        "Ative em Ajustes"
    ],
    BodySettings => [
        "Abre Ajustes > Privacidad y seguridad > Modo de desarrollador, actívalo y toca Reiniciar cuando el iPhone te lo pida.",
        "Open Settings > Privacy & Security > Developer Mode, turn it on and tap Restart when the iPhone asks.",
        "Abra Ajustes > Privacidade e Segurança > Modo de Desenvolvedor, ative-o e toque em Reiniciar quando o iPhone pedir."
    ],
    TitleRebooting => ["Reiniciando...", "Restarting...", "Reiniciando..."],
    BodyRebooting => [
        "Espera a que el iPhone vuelva a encender. La conexión regresa sola.",
        "Wait for the iPhone to turn back on. The connection returns on its own.",
        "Aguarde o iPhone ligar novamente. A conexão volta sozinha."
    ],
    TitleLocked => [
        "Desbloquea el iPhone",
        "Unlock the iPhone",
        "Desbloqueie o iPhone"
    ],
    BodyLocked => [
        "Después de reiniciar, el iPhone necesita tu código. Ingrésalo en el iPhone y esta pantalla seguirá sola.",
        "After restarting, the iPhone needs your passcode. Enter it on the iPhone and this screen will continue on its own.",
        "Após reiniciar, o iPhone precisa do seu código. Digite-o no iPhone e esta tela continuará sozinha."
    ],
    TitleConfirm => [
        "Falta tu confirmación",
        "Your confirmation is needed",
        "Falta sua confirmação"
    ],
    BodyConfirm => [
        "Confirma en el iPhone: toca Activar e ingresa tu código",
        "Confirm on the iPhone: tap Turn On and enter your passcode",
        "Confirme no iPhone: toque em Ativar e digite seu código"
    ],
    TitleSuccess => [
        "Modo desarrollador activo",
        "Developer Mode is on",
        "Modo de desenvolvedor ativo"
    ],
    BodySuccess => ["Todo listo.", "All set.", "Tudo pronto."],

    ErrTitle => ["Algo salió mal", "Something went wrong", "Algo deu errado"],
    ErrLocked => [
        "Desbloquea el iPhone y toca Reintentar.",
        "Unlock the iPhone and tap Retry.",
        "Desbloqueie o iPhone e toque em Tentar novamente."
    ],
    ErrReadFailed => [
        "No se pudo leer el iPhone ({detail}).",
        "Could not read the iPhone ({detail}).",
        "Não foi possível ler o iPhone ({detail})."
    ],
    ManualHelp => [
        "Abre Ajustes > Privacidad y seguridad > Modo de desarrollador, actívalo y toca Reiniciar cuando el iPhone te lo pida. Luego toca Reintentar.",
        "Open Settings > Privacy & Security > Developer Mode, turn it on and tap Restart when the iPhone asks. Then tap Retry.",
        "Abra Ajustes > Privacidade e Segurança > Modo de Desenvolvedor, ative-o e toque em Reiniciar quando o iPhone pedir. Depois toque em Tentar novamente."
    ],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_translated() {
        for &k in Key::ALL {
            for l in Lang::ALL {
                assert!(!t(l, k).is_empty(), "{k:?} empty in {l:?}");
            }
        }
    }

    #[test]
    fn locale_parsing() {
        for (s, l) in [
            ("en_US.UTF-8", Lang::En),
            ("pt_BR", Lang::Pt),
            ("es_CO", Lang::Es),
            ("", Lang::Es),
            ("C", Lang::Es),
            ("EN_gb", Lang::En),
        ] {
            assert_eq!(Lang::from_locale(s), l, "{s}");
        }
    }
}
