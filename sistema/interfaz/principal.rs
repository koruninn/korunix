//! PARTE INTERNA DE KORUNIX.
//!
//! Esta interfaz presenta información y solicita acciones al motor público
//! `korunix`. No evalúa Nix ni mantiene una segunda implementación del sistema.

use adw::prelude::*;
use adw::{gio, glib};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::OnceLock;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, TryRecvError},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APPLICATION_ID: &str = "io.github.koruninn.Korunix";

#[derive(Clone, Copy)]
enum Idioma {
    BelarusLatino,
    Belarus,
    Catalan,
    Checo,
    Aleman,
    Ingles,
    Espanol,
    Frances,
    Gallego,
    Hungaro,
    Italiano,
    Coreano,
    Kurdo,
    Neerlandes,
    NoruegoNynorsk,
    Polaco,
    PortuguesBrasil,
    Ruso,
    Sueco,
    Turco,
    Ucraniano,
    Vietnamita,
    ChinoSimplificado,
}

fn idioma_actual() -> Idioma {
    let valor = env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .replace('-', "_");

    if valor.starts_with("be_latn") {
        Idioma::BelarusLatino
    } else if valor.starts_with("be") {
        Idioma::Belarus
    } else if valor.starts_with("ca") {
        Idioma::Catalan
    } else if valor.starts_with("cs") {
        Idioma::Checo
    } else if valor.starts_with("de") {
        Idioma::Aleman
    } else if valor.starts_with("en") {
        Idioma::Ingles
    } else if valor.starts_with("es") {
        Idioma::Espanol
    } else if valor.starts_with("fr") {
        Idioma::Frances
    } else if valor.starts_with("gl") {
        Idioma::Gallego
    } else if valor.starts_with("hu") {
        Idioma::Hungaro
    } else if valor.starts_with("it") {
        Idioma::Italiano
    } else if valor.starts_with("ko") {
        Idioma::Coreano
    } else if valor.starts_with("ku") {
        Idioma::Kurdo
    } else if valor.starts_with("nl") {
        Idioma::Neerlandes
    } else if valor.starts_with("nn") {
        Idioma::NoruegoNynorsk
    } else if valor.starts_with("pl") {
        Idioma::Polaco
    } else if valor.starts_with("pt_br") {
        Idioma::PortuguesBrasil
    } else if valor.starts_with("ru") {
        Idioma::Ruso
    } else if valor.starts_with("sv") {
        Idioma::Sueco
    } else if valor.starts_with("tr") {
        Idioma::Turco
    } else if valor.starts_with("uk") {
        Idioma::Ucraniano
    } else if valor.starts_with("vi") {
        Idioma::Vietnamita
    } else if valor.starts_with("zh") {
        Idioma::ChinoSimplificado
    } else {
        Idioma::Espanol
    }
}

fn idioma_tag(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::BelarusLatino => "be-Latn",
        Idioma::Belarus => "be",
        Idioma::Catalan => "ca",
        Idioma::Checo => "cs",
        Idioma::Aleman => "de",
        Idioma::Ingles => "en",
        Idioma::Espanol => "es",
        Idioma::Frances => "fr",
        Idioma::Gallego => "gl-ES",
        Idioma::Hungaro => "hu",
        Idioma::Italiano => "it",
        Idioma::Coreano => "ko",
        Idioma::Kurdo => "ku",
        Idioma::Neerlandes => "nl",
        Idioma::NoruegoNynorsk => "nn",
        Idioma::Polaco => "pl",
        Idioma::PortuguesBrasil => "pt-BR",
        Idioma::Ruso => "ru",
        Idioma::Sueco => "sv",
        Idioma::Turco => "tr",
        Idioma::Ucraniano => "uk-UA",
        Idioma::Vietnamita => "vi",
        Idioma::ChinoSimplificado => "zh-Hans",
    }
}

fn catalogo_visible_raw(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::BelarusLatino => include_str!("i18n/be-Latn.json"),
        Idioma::Belarus => include_str!("i18n/be.json"),
        Idioma::Catalan => include_str!("i18n/ca.json"),
        Idioma::Checo => include_str!("i18n/cs.json"),
        Idioma::Aleman => include_str!("i18n/de.json"),
        Idioma::Ingles => include_str!("i18n/en.json"),
        Idioma::Espanol => include_str!("i18n/es.json"),
        Idioma::Frances => include_str!("i18n/fr.json"),
        Idioma::Gallego => include_str!("i18n/gl-ES.json"),
        Idioma::Hungaro => include_str!("i18n/hu.json"),
        Idioma::Italiano => include_str!("i18n/it.json"),
        Idioma::Coreano => include_str!("i18n/ko.json"),
        Idioma::Kurdo => include_str!("i18n/ku.json"),
        Idioma::Neerlandes => include_str!("i18n/nl.json"),
        Idioma::NoruegoNynorsk => include_str!("i18n/nn.json"),
        Idioma::Polaco => include_str!("i18n/pl.json"),
        Idioma::PortuguesBrasil => include_str!("i18n/pt-BR.json"),
        Idioma::Ruso => include_str!("i18n/ru.json"),
        Idioma::Sueco => include_str!("i18n/sv.json"),
        Idioma::Turco => include_str!("i18n/tr.json"),
        Idioma::Ucraniano => include_str!("i18n/uk-UA.json"),
        Idioma::Vietnamita => include_str!("i18n/vi.json"),
        Idioma::ChinoSimplificado => include_str!("i18n/zh-Hans.json"),
    }
}

fn catalogo_visible(idioma: Idioma) -> &'static HashMap<String, String> {
    static CATALOGOS: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

    let catalogos = CATALOGOS.get_or_init(|| {
        let mut resultado = HashMap::new();
        for idioma in [
            Idioma::BelarusLatino,
            Idioma::Belarus,
            Idioma::Catalan,
            Idioma::Checo,
            Idioma::Aleman,
            Idioma::Ingles,
            Idioma::Espanol,
            Idioma::Frances,
            Idioma::Gallego,
            Idioma::Hungaro,
            Idioma::Italiano,
            Idioma::Coreano,
            Idioma::Kurdo,
            Idioma::Neerlandes,
            Idioma::NoruegoNynorsk,
            Idioma::Polaco,
            Idioma::PortuguesBrasil,
            Idioma::Ruso,
            Idioma::Sueco,
            Idioma::Turco,
            Idioma::Ucraniano,
            Idioma::Vietnamita,
            Idioma::ChinoSimplificado,
        ] {
            let catalogo =
                serde_json::from_str::<HashMap<String, String>>(catalogo_visible_raw(idioma))
                    .unwrap_or_default();
            resultado.insert(idioma_tag(idioma), catalogo);
        }
        resultado
    });

    catalogos
        .get(idioma_tag(idioma))
        .expect("el catálogo de la localización debe existir")
}

fn aplicar_plantilla_visible(origen: &str, destino: &str, texto: &str) -> Option<String> {
    if !origen.contains('{') {
        return None;
    }

    let mut capturas = Vec::<(String, String)>::new();
    let mut origen_pos = 0usize;
    let mut texto_pos = 0usize;

    while let Some(rel_inicio) = origen[origen_pos..].find('{') {
        let inicio = origen_pos + rel_inicio;
        let rel_fin = origen[inicio..].find('}')?;
        let fin = inicio + rel_fin;
        let fijo = &origen[origen_pos..inicio];

        if !texto[texto_pos..].starts_with(fijo) {
            return None;
        }
        texto_pos += fijo.len();

        let token = &origen[inicio..=fin];
        let despues = fin + 1;
        let siguiente_inicio = origen[despues..]
            .find('{')
            .map(|valor| despues + valor)
            .unwrap_or(origen.len());
        let siguiente_fijo = &origen[despues..siguiente_inicio];

        let valor_fin = if siguiente_fijo.is_empty() {
            texto.len()
        } else {
            texto[texto_pos..].find(siguiente_fijo)? + texto_pos
        };

        capturas.push((token.to_string(), texto[texto_pos..valor_fin].to_string()));
        texto_pos = valor_fin;
        origen_pos = despues;
    }

    let final_fijo = &origen[origen_pos..];
    if !texto[texto_pos..].starts_with(final_fijo) || texto_pos + final_fijo.len() != texto.len() {
        return None;
    }

    let mut resultado = destino.to_string();
    for (token, valor) in capturas {
        resultado = resultado.replace(&token, &valor);
    }
    Some(resultado)
}

fn localizar_visible(idioma: Idioma, texto: impl AsRef<str>) -> String {
    let texto = texto.as_ref();
    let catalogo = catalogo_visible(idioma);

    if let Some(exacto) = catalogo.get(texto) {
        return exacto.clone();
    }

    for (origen, destino) in catalogo {
        if let Some(resultado) = aplicar_plantilla_visible(origen, destino, texto) {
            return resultado;
        }
    }

    texto.to_string()
}

fn texto(idioma: Idioma, clave: &str) -> &'static str {
    match (idioma, clave) {
        (Idioma::Ingles, "subtitle") => "NixOS control center",
        (Idioma::BelarusLatino, "subtitle") => "Centr kiravannja nixos",
        (Idioma::Belarus, "subtitle") => "Цэнтр кіравання NixOS",
        (Idioma::Catalan, "subtitle") => "Centre de control NixOS",
        (Idioma::Checo, "subtitle") => "Ovládací centrum NixOS",
        (Idioma::Aleman, "subtitle") => "NixOS-Kontrollzentrum",
        (Idioma::Frances, "subtitle") => "Centre de contrôle NixOS",
        (Idioma::Gallego, "subtitle") => "Centro de control NixOS",
        (Idioma::Italiano, "subtitle") => "Centro di controllo NixOS",
        (Idioma::Coreano, "subtitle") => "NixOS 제어 센터",
        (Idioma::Kurdo, "subtitle") => "Navenda kontrolê ya NixOS",
        (Idioma::Neerlandes, "subtitle") => "NixOS-controlecentrum",
        (Idioma::NoruegoNynorsk, "subtitle") => "NixOS kontrollsenter",
        (Idioma::Polaco, "subtitle") => "Centrum kontroli NixOS",
        (Idioma::PortuguesBrasil, "subtitle") => "Centro de controle NixOS",
        (Idioma::Ruso, "subtitle") => "Центр управления NixOS",
        (Idioma::Sueco, "subtitle") => "NixOS kontrollcenter",
        (Idioma::Turco, "subtitle") => "NixOS kontrol merkezi",
        (Idioma::Ucraniano, "subtitle") => "Центр керування NixOS",
        (Idioma::Vietnamita, "subtitle") => "Trung tâm điều khiển NixOS",
        (Idioma::ChinoSimplificado, "subtitle") => "NixOS 控制中心",
        (Idioma::Ingles, "summary") => "Summary",
        (Idioma::BelarusLatino, "summary") => "Rezjume",
        (Idioma::Belarus, "summary") => "Рэзюмэ",
        (Idioma::Catalan, "summary") => "Resum",
        (Idioma::Checo, "summary") => "Shrnutí",
        (Idioma::Aleman, "summary") => "Zusammenfassung",
        (Idioma::Frances, "summary") => "Résumé",
        (Idioma::Gallego, "summary") => "Resumo",
        (Idioma::Italiano, "summary") => "Riepilogo",
        (Idioma::Coreano, "summary") => "요약",
        (Idioma::Kurdo, "summary") => "Kurte",
        (Idioma::Neerlandes, "summary") => "Samenvatting",
        (Idioma::NoruegoNynorsk, "summary") => "Sammendrag",
        (Idioma::Polaco, "summary") => "Podsumowanie",
        (Idioma::PortuguesBrasil, "summary") => "Resumo",
        (Idioma::Ruso, "summary") => "Резюме",
        (Idioma::Sueco, "summary") => "Sammanfattning",
        (Idioma::Turco, "summary") => "Özet",
        (Idioma::Ucraniano, "summary") => "Резюме",
        (Idioma::Vietnamita, "summary") => "Tóm tắt",
        (Idioma::ChinoSimplificado, "summary") => "摘要",
        (Idioma::Ingles, "updates") => "Updates",
        (Idioma::BelarusLatino, "updates") => "Abnaŭłjenni",
        (Idioma::Belarus, "updates") => "Абнаўленні",
        (Idioma::Catalan, "updates") => "Actualitzacions",
        (Idioma::Checo, "updates") => "Aktualizace",
        (Idioma::Aleman, "updates") => "Aktualisierungen",
        (Idioma::Frances, "updates") => "Mises à jour",
        (Idioma::Gallego, "updates") => "Actualizacións",
        (Idioma::Italiano, "updates") => "Aggiornamenti",
        (Idioma::Coreano, "updates") => "업데이트",
        (Idioma::Kurdo, "updates") => "Nûvekirin",
        (Idioma::Neerlandes, "updates") => "Updates",
        (Idioma::NoruegoNynorsk, "updates") => "Oppdateringer",
        (Idioma::Polaco, "updates") => "Aktualizacje",
        (Idioma::PortuguesBrasil, "updates") => "Atualizações",
        (Idioma::Ruso, "updates") => "Обновления",
        (Idioma::Sueco, "updates") => "Uppdateringar",
        (Idioma::Turco, "updates") => "Güncellemeler",
        (Idioma::Ucraniano, "updates") => "Оновлення",
        (Idioma::Vietnamita, "updates") => "Cập nhật",
        (Idioma::ChinoSimplificado, "updates") => "更新",
        (Idioma::Ingles, "localization") => "Language and region",
        (Idioma::BelarusLatino, "localization") => "Mova i rehijon",
        (Idioma::Belarus, "localization") => "Мова і рэгіён",
        (Idioma::Catalan, "localization") => "Idioma i regió",
        (Idioma::Checo, "localization") => "Jazyk a region",
        (Idioma::Aleman, "localization") => "Sprache und Region",
        (Idioma::Frances, "localization") => "Langue et région",
        (Idioma::Gallego, "localization") => "Idioma e rexión",
        (Idioma::Italiano, "localization") => "Lingua e regione",
        (Idioma::Coreano, "localization") => "언어 및 지역",
        (Idioma::Kurdo, "localization") => "Ziman û herêm",
        (Idioma::Neerlandes, "localization") => "Taal en regio",
        (Idioma::NoruegoNynorsk, "localization") => "Språk og region",
        (Idioma::Polaco, "localization") => "Język i region",
        (Idioma::PortuguesBrasil, "localization") => "Idioma e região",
        (Idioma::Ruso, "localization") => "Язык и регион",
        (Idioma::Sueco, "localization") => "Språk och region",
        (Idioma::Turco, "localization") => "Dil ve bölge",
        (Idioma::Ucraniano, "localization") => "Мова та регіон",
        (Idioma::Vietnamita, "localization") => "Ngôn ngữ và khu vực",
        (Idioma::ChinoSimplificado, "localization") => "语言和地区",
        (Idioma::Ingles, "hardware") => "Hardware",
        (Idioma::BelarusLatino, "hardware") => "Abstałjavannje",
        (Idioma::Belarus, "hardware") => "Абсталяванне",
        (Idioma::Catalan, "hardware") => "Maquinari",
        (Idioma::Checo, "hardware") => "Hardware",
        (Idioma::Aleman, "hardware") => "Hardware",
        (Idioma::Frances, "hardware") => "Matériel",
        (Idioma::Gallego, "hardware") => "Hardware",
        (Idioma::Italiano, "hardware") => "Ferramenta",
        (Idioma::Coreano, "hardware") => "하드웨어",
        (Idioma::Kurdo, "hardware") => "Hardware",
        (Idioma::Neerlandes, "hardware") => "Hardware",
        (Idioma::NoruegoNynorsk, "hardware") => "Maskinvare",
        (Idioma::Polaco, "hardware") => "Sprzęt",
        (Idioma::PortuguesBrasil, "hardware") => "Hardware",
        (Idioma::Ruso, "hardware") => "Аппаратное обеспечение",
        (Idioma::Sueco, "hardware") => "Hårdvara",
        (Idioma::Turco, "hardware") => "Donanım",
        (Idioma::Ucraniano, "hardware") => "Обладнання",
        (Idioma::Vietnamita, "hardware") => "Phần cứng",
        (Idioma::ChinoSimplificado, "hardware") => "硬件",
        (Idioma::Ingles, "people") => "People",
        (Idioma::BelarusLatino, "people") => "Łjudzi",
        (Idioma::Belarus, "people") => "Людзі",
        (Idioma::Catalan, "people") => "Persones",
        (Idioma::Checo, "people") => "Lidé",
        (Idioma::Aleman, "people") => "Menschen",
        (Idioma::Frances, "people") => "Personnes",
        (Idioma::Gallego, "people") => "Persoas",
        (Idioma::Italiano, "people") => "Persone",
        (Idioma::Coreano, "people") => "사람",
        (Idioma::Kurdo, "people") => "Kes",
        (Idioma::Neerlandes, "people") => "Mensen",
        (Idioma::NoruegoNynorsk, "people") => "Folk",
        (Idioma::Polaco, "people") => "Osoby",
        (Idioma::PortuguesBrasil, "people") => "Pessoas",
        (Idioma::Ruso, "people") => "Люди",
        (Idioma::Sueco, "people") => "Folk",
        (Idioma::Turco, "people") => "İnsan",
        (Idioma::Ucraniano, "people") => "Люди",
        (Idioma::Vietnamita, "people") => "Mọi người",
        (Idioma::ChinoSimplificado, "people") => "人物",
        (Idioma::Ingles, "refresh") => "Refresh",
        (Idioma::BelarusLatino, "refresh") => "Abnavic",
        (Idioma::Belarus, "refresh") => "Абнавіць",
        (Idioma::Catalan, "refresh") => "Actualitza",
        (Idioma::Checo, "refresh") => "Obnovit",
        (Idioma::Aleman, "refresh") => "Aktualisieren",
        (Idioma::Frances, "refresh") => "Actualiser",
        (Idioma::Gallego, "refresh") => "Actualizar",
        (Idioma::Italiano, "refresh") => "Aggiorna",
        (Idioma::Coreano, "refresh") => "새로 고침",
        (Idioma::Kurdo, "refresh") => "Nûvekirin",
        (Idioma::Neerlandes, "refresh") => "Vernieuwen",
        (Idioma::NoruegoNynorsk, "refresh") => "Oppdater",
        (Idioma::Polaco, "refresh") => "Odśwież",
        (Idioma::PortuguesBrasil, "refresh") => "Atualizar",
        (Idioma::Ruso, "refresh") => "Обновить",
        (Idioma::Sueco, "refresh") => "Uppdatera",
        (Idioma::Turco, "refresh") => "Yenile",
        (Idioma::Ucraniano, "refresh") => "Оновити",
        (Idioma::Vietnamita, "refresh") => "Làm mới",
        (Idioma::ChinoSimplificado, "refresh") => "刷新",
        (Idioma::Ingles, "channel") => "System channel",
        (Idioma::BelarusLatino, "channel") => "Sistemny kanał",
        (Idioma::Belarus, "channel") => "Сістэмны канал",
        (Idioma::Catalan, "channel") => "Canal del sistema",
        (Idioma::Checo, "channel") => "Systémový kanál",
        (Idioma::Aleman, "channel") => "Systemkanal",
        (Idioma::Frances, "channel") => "Canal système",
        (Idioma::Gallego, "channel") => "Canle do sistema",
        (Idioma::Italiano, "channel") => "Canale di sistema",
        (Idioma::Coreano, "channel") => "시스템 채널",
        (Idioma::Kurdo, "channel") => "Kanala pergalê",
        (Idioma::Neerlandes, "channel") => "Systeemkanaal",
        (Idioma::NoruegoNynorsk, "channel") => "Systemkanal",
        (Idioma::Polaco, "channel") => "Kanał systemowy",
        (Idioma::PortuguesBrasil, "channel") => "Canal do sistema",
        (Idioma::Ruso, "channel") => "Системный канал",
        (Idioma::Sueco, "channel") => "Systemkanal",
        (Idioma::Turco, "channel") => "Sistem kanalı",
        (Idioma::Ucraniano, "channel") => "Системний канал",
        (Idioma::Vietnamita, "channel") => "Kênh hệ thống",
        (Idioma::ChinoSimplificado, "channel") => "系统通道",
        (Idioma::Ingles, "stable") => "Stable",
        (Idioma::BelarusLatino, "stable") => "Stabiłny",
        (Idioma::Belarus, "stable") => "Стабільны",
        (Idioma::Catalan, "stable") => "Estable",
        (Idioma::Checo, "stable") => "Stabilní",
        (Idioma::Aleman, "stable") => "Stabil",
        (Idioma::Frances, "stable") => "Stable",
        (Idioma::Gallego, "stable") => "Estable",
        (Idioma::Italiano, "stable") => "Stabile",
        (Idioma::Coreano, "stable") => "안정적",
        (Idioma::Kurdo, "stable") => "Stable",
        (Idioma::Neerlandes, "stable") => "Stabiel",
        (Idioma::NoruegoNynorsk, "stable") => "Stabil",
        (Idioma::Polaco, "stable") => "Stabilny",
        (Idioma::PortuguesBrasil, "stable") => "Estável",
        (Idioma::Ruso, "stable") => "Стабильный",
        (Idioma::Sueco, "stable") => "Stabil",
        (Idioma::Turco, "stable") => "Kararlı",
        (Idioma::Ucraniano, "stable") => "Стабільний",
        (Idioma::Vietnamita, "stable") => "Ổn định",
        (Idioma::ChinoSimplificado, "stable") => "稳定",
        (Idioma::Ingles, "unstable") => "Unstable",
        (Idioma::BelarusLatino, "unstable") => "Njaŭstojłivy",
        (Idioma::Belarus, "unstable") => "Нестабільны",
        (Idioma::Catalan, "unstable") => "Inestable",
        (Idioma::Checo, "unstable") => "Nestabilní",
        (Idioma::Aleman, "unstable") => "Instabil",
        (Idioma::Frances, "unstable") => "Instable",
        (Idioma::Gallego, "unstable") => "Inestable",
        (Idioma::Italiano, "unstable") => "Instabile",
        (Idioma::Coreano, "unstable") => "불안정",
        (Idioma::Kurdo, "unstable") => "Bêîstîqrar",
        (Idioma::Neerlandes, "unstable") => "Instabiel",
        (Idioma::NoruegoNynorsk, "unstable") => "Ustabil",
        (Idioma::Polaco, "unstable") => "Niestabilny",
        (Idioma::PortuguesBrasil, "unstable") => "Instável",
        (Idioma::Ruso, "unstable") => "Нестабильный",
        (Idioma::Sueco, "unstable") => "Instabil",
        (Idioma::Turco, "unstable") => "Kararsız",
        (Idioma::Ucraniano, "unstable") => "Нестабільний",
        (Idioma::Vietnamita, "unstable") => "Không ổn định",
        (Idioma::ChinoSimplificado, "unstable") => "不稳定",
        (Idioma::Ingles, "prepare") => "Prepare change",
        (Idioma::BelarusLatino, "prepare") => "Rychtujcje zmjenu",
        (Idioma::Belarus, "prepare") => "Падрыхтуйце змену",
        (Idioma::Catalan, "prepare") => "Prepara el canvi",
        (Idioma::Checo, "prepare") => "Připravte změnu",
        (Idioma::Aleman, "prepare") => "Änderung vorbereiten",
        (Idioma::Frances, "prepare") => "Préparer le changement",
        (Idioma::Gallego, "prepare") => "Preparar o cambio",
        (Idioma::Italiano, "prepare") => "Prepara la modifica",
        (Idioma::Coreano, "prepare") => "변경 준비",
        (Idioma::Kurdo, "prepare") => "Guherînê amade bike",
        (Idioma::Neerlandes, "prepare") => "Wijziging voorbereiden",
        (Idioma::NoruegoNynorsk, "prepare") => "Forbered endring",
        (Idioma::Polaco, "prepare") => "Przygotuj zmianę",
        (Idioma::PortuguesBrasil, "prepare") => "Preparar a mudança",
        (Idioma::Ruso, "prepare") => "Подготовить изменение",
        (Idioma::Sueco, "prepare") => "Förbered förändring",
        (Idioma::Turco, "prepare") => "Değişikliği hazırla",
        (Idioma::Ucraniano, "prepare") => "Підготуйте зміни",
        (Idioma::Vietnamita, "prepare") => "Chuẩn bị thay đổi",
        (Idioma::ChinoSimplificado, "prepare") => "准备零钱",
        (Idioma::Ingles, "configured") => "Configured",
        (Idioma::BelarusLatino, "configured") => "Naładžana",
        (Idioma::Belarus, "configured") => "Наладжаны",
        (Idioma::Catalan, "configured") => "Configurat",
        (Idioma::Checo, "configured") => "Nakonfigurováno",
        (Idioma::Aleman, "configured") => "Konfiguriert",
        (Idioma::Frances, "configured") => "Configuré",
        (Idioma::Gallego, "configured") => "Configurado",
        (Idioma::Italiano, "configured") => "Configurato",
        (Idioma::Coreano, "configured") => "구성됨",
        (Idioma::Kurdo, "configured") => "Mîheng kirin",
        (Idioma::Neerlandes, "configured") => "Geconfigureerd",
        (Idioma::NoruegoNynorsk, "configured") => "Konfigurert",
        (Idioma::Polaco, "configured") => "Skonfigurowano",
        (Idioma::PortuguesBrasil, "configured") => "Configurado",
        (Idioma::Ruso, "configured") => "Настроено",
        (Idioma::Sueco, "configured") => "Konfigurerad",
        (Idioma::Turco, "configured") => "Yapılandırılmış",
        (Idioma::Ucraniano, "configured") => "Налаштовано",
        (Idioma::Vietnamita, "configured") => "Đã định cấu hình",
        (Idioma::ChinoSimplificado, "configured") => "已配置",
        (Idioma::Ingles, "target_channel") => "New channel",
        (Idioma::BelarusLatino, "target_channel") => "Novy kanał",
        (Idioma::Belarus, "target_channel") => "Новы канал",
        (Idioma::Catalan, "target_channel") => "Canal nou",
        (Idioma::Checo, "target_channel") => "Nový kanál",
        (Idioma::Aleman, "target_channel") => "Neuer Kanal",
        (Idioma::Frances, "target_channel") => "Nouvelle chaîne",
        (Idioma::Gallego, "target_channel") => "Nova canle",
        (Idioma::Italiano, "target_channel") => "Nuovo canale",
        (Idioma::Coreano, "target_channel") => "새 채널",
        (Idioma::Kurdo, "target_channel") => "Kanala nû",
        (Idioma::Neerlandes, "target_channel") => "Nieuw kanaal",
        (Idioma::NoruegoNynorsk, "target_channel") => "Ny kanal",
        (Idioma::Polaco, "target_channel") => "Nowy kanał",
        (Idioma::PortuguesBrasil, "target_channel") => "Novo canal",
        (Idioma::Ruso, "target_channel") => "Новый канал",
        (Idioma::Sueco, "target_channel") => "Ny kanal",
        (Idioma::Turco, "target_channel") => "Yeni kanal",
        (Idioma::Ucraniano, "target_channel") => "Новий канал",
        (Idioma::Vietnamita, "target_channel") => "Kênh mới",
        (Idioma::ChinoSimplificado, "target_channel") => "新频道",
        (Idioma::Ingles, "change") => "Change",
        (Idioma::BelarusLatino, "change") => "Zmjena",
        (Idioma::Belarus, "change") => "Змена",
        (Idioma::Catalan, "change") => "Canvi",
        (Idioma::Checo, "change") => "Změna",
        (Idioma::Aleman, "change") => "Änderung",
        (Idioma::Frances, "change") => "Modification",
        (Idioma::Gallego, "change") => "Cambiar",
        (Idioma::Italiano, "change") => "Modifica",
        (Idioma::Coreano, "change") => "변경",
        (Idioma::Kurdo, "change") => "Guhertin",
        (Idioma::Neerlandes, "change") => "Wijziging",
        (Idioma::NoruegoNynorsk, "change") => "Endre",
        (Idioma::Polaco, "change") => "Zmień",
        (Idioma::PortuguesBrasil, "change") => "Alteração",
        (Idioma::Ruso, "change") => "Изменить",
        (Idioma::Sueco, "change") => "Ändra",
        (Idioma::Turco, "change") => "Değiştir",
        (Idioma::Ucraniano, "change") => "Змінити",
        (Idioma::Vietnamita, "change") => "Thay đổi",
        (Idioma::ChinoSimplificado, "change") => "更改",
        (Idioma::Ingles, "confirm_channel") => "Confirm channel change",
        (Idioma::BelarusLatino, "confirm_channel") => "Pacvjerdzicje zmjenu kanała",
        (Idioma::Belarus, "confirm_channel") => "Пацвердзіць змену канала",
        (Idioma::Catalan, "confirm_channel") => "Confirmeu el canvi de canal",
        (Idioma::Checo, "confirm_channel") => "Potvrďte změnu kanálu",
        (Idioma::Aleman, "confirm_channel") => "Kanalwechsel bestätigen",
        (Idioma::Frances, "confirm_channel") => "Confirmer le changement de chaîne",
        (Idioma::Gallego, "confirm_channel") => "Confirmar o cambio de canle",
        (Idioma::Italiano, "confirm_channel") => "Conferma cambio canale",
        (Idioma::Coreano, "confirm_channel") => "채널 변경 확인",
        (Idioma::Kurdo, "confirm_channel") => "Guhertina kanalê piştrast bike",
        (Idioma::Neerlandes, "confirm_channel") => "Kanaalwijziging bevestigen",
        (Idioma::NoruegoNynorsk, "confirm_channel") => "Bekreft kanalbytte",
        (Idioma::Polaco, "confirm_channel") => "Potwierdź zmianę kanału",
        (Idioma::PortuguesBrasil, "confirm_channel") => "Confirme a mudança de canal",
        (Idioma::Ruso, "confirm_channel") => "Подтвердить смену канала",
        (Idioma::Sueco, "confirm_channel") => "Bekräfta kanalbyte",
        (Idioma::Turco, "confirm_channel") => "Kanal değişikliğini onayla",
        (Idioma::Ucraniano, "confirm_channel") => "Підтвердити зміну каналу",
        (Idioma::Vietnamita, "confirm_channel") => "Xác nhận thay đổi kênh",
        (Idioma::ChinoSimplificado, "confirm_channel") => "确认频道变更",
        (Idioma::Ingles, "cancel") => "Cancel",
        (Idioma::BelarusLatino, "cancel") => "Admjanic",
        (Idioma::Belarus, "cancel") => "Скасаваць",
        (Idioma::Catalan, "cancel") => "Cancel·la",
        (Idioma::Checo, "cancel") => "Zrušit",
        (Idioma::Aleman, "cancel") => "Abbrechen",
        (Idioma::Frances, "cancel") => "Annuler",
        (Idioma::Gallego, "cancel") => "Cancelar",
        (Idioma::Italiano, "cancel") => "Annulla",
        (Idioma::Coreano, "cancel") => "취소",
        (Idioma::Kurdo, "cancel") => "Têk bibe",
        (Idioma::Neerlandes, "cancel") => "Annuleren",
        (Idioma::NoruegoNynorsk, "cancel") => "Avbryt",
        (Idioma::Polaco, "cancel") => "Anuluj",
        (Idioma::PortuguesBrasil, "cancel") => "Cancelar",
        (Idioma::Ruso, "cancel") => "Отменить",
        (Idioma::Sueco, "cancel") => "Avbryt",
        (Idioma::Turco, "cancel") => "İptal",
        (Idioma::Ucraniano, "cancel") => "Скасувати",
        (Idioma::Vietnamita, "cancel") => "Hủy",
        (Idioma::ChinoSimplificado, "cancel") => "取消",
        (Idioma::Ingles, "apply_change") => "Apply change",
        (Idioma::BelarusLatino, "apply_change") => "Užyc zmjeny",
        (Idioma::Belarus, "apply_change") => "Прымяніць змены",
        (Idioma::Catalan, "apply_change") => "Aplica el canvi",
        (Idioma::Checo, "apply_change") => "Použít změnu",
        (Idioma::Aleman, "apply_change") => "Änderung übernehmen",
        (Idioma::Frances, "apply_change") => "Appliquer la modification",
        (Idioma::Gallego, "apply_change") => "Aplicar o cambio",
        (Idioma::Italiano, "apply_change") => "Applica la modifica",
        (Idioma::Coreano, "apply_change") => "변경 사항 적용",
        (Idioma::Kurdo, "apply_change") => "Guherînê bi kar bîne",
        (Idioma::Neerlandes, "apply_change") => "Wijziging toepassen",
        (Idioma::NoruegoNynorsk, "apply_change") => "Bruk endring",
        (Idioma::Polaco, "apply_change") => "Zastosuj zmianę",
        (Idioma::PortuguesBrasil, "apply_change") => "Aplicar alteração",
        (Idioma::Ruso, "apply_change") => "Применить изменение",
        (Idioma::Sueco, "apply_change") => "Tillämpa ändring",
        (Idioma::Turco, "apply_change") => "Değişikliği uygula",
        (Idioma::Ucraniano, "apply_change") => "Застосувати зміни",
        (Idioma::Vietnamita, "apply_change") => "Áp dụng thay đổi",
        (Idioma::ChinoSimplificado, "apply_change") => "应用更改",
        (Idioma::Ingles, "no_change") => "Choose a different channel first.",
        (Idioma::BelarusLatino, "no_change") => "Spačatku vybjerycje inšy kanał.",
        (Idioma::Belarus, "no_change") => "Спачатку абярыце іншы канал.",
        (Idioma::Catalan, "no_change") => "Trieu primer un canal diferent.",
        (Idioma::Checo, "no_change") => "Nejprve vyberte jiný kanál.",
        (Idioma::Aleman, "no_change") => "Wählen Sie zunächst einen anderen Kanal.",
        (Idioma::Frances, "no_change") => "Choisissez d'abord un autre canal.",
        (Idioma::Gallego, "no_change") => "Escolla primeiro unha canle diferente.",
        (Idioma::Italiano, "no_change") => "Scegli prima un canale diverso.",
        (Idioma::Coreano, "no_change") => "먼저 다른 채널을 선택하세요.",
        (Idioma::Kurdo, "no_change") => "Pêşî kanaleke cuda hilbijêre.",
        (Idioma::Neerlandes, "no_change") => "Kies eerst een ander kanaal.",
        (Idioma::NoruegoNynorsk, "no_change") => "Velg en annen kanal først.",
        (Idioma::Polaco, "no_change") => "Najpierw wybierz inny kanał.",
        (Idioma::PortuguesBrasil, "no_change") => "Escolha primeiro um canal diferente.",
        (Idioma::Ruso, "no_change") => "Сначала выберите другой канал.",
        (Idioma::Sueco, "no_change") => "Välj en annan kanal först.",
        (Idioma::Turco, "no_change") => "Choose a different channel first.",
        (Idioma::Ucraniano, "no_change") => "Спочатку виберіть інший канал.",
        (Idioma::Vietnamita, "no_change") => "Trước tiên hãy chọn một kênh khác.",
        (Idioma::ChinoSimplificado, "no_change") => "首先选择不同的频道。",
        (Idioma::Ingles, "plan_failed") => "The change preview could not be prepared.",
        (Idioma::BelarusLatino, "plan_failed") => "Nje ŭdałosja padrychtavac papjaredni prahłjad zmjanjennjaŭ.",
        (Idioma::Belarus, "plan_failed") => "Не ўдалося падрыхтаваць папярэдні прагляд змяненняў.",
        (Idioma::Catalan, "plan_failed") => "No s'ha pogut preparar la previsualització del canvi.",
        (Idioma::Checo, "plan_failed") => "Náhled změny nelze připravit.",
        (Idioma::Aleman, "plan_failed") => "Die Änderungsvorschau konnte nicht vorbereitet werden.",
        (Idioma::Frances, "plan_failed") => "L'aperçu des modifications n'a pas pu être préparé.",
        (Idioma::Gallego, "plan_failed") => "Non se puido preparar a vista previa do cambio.",
        (Idioma::Italiano, "plan_failed") => "Impossibile preparare l'anteprima della modifica.",
        (Idioma::Coreano, "plan_failed") => "변경 미리보기를 준비할 수 없습니다.",
        (Idioma::Kurdo, "plan_failed") => "Pêşdîtina guherînê nehat amadekirin.",
        (Idioma::Neerlandes, "plan_failed") => "Het wijzigingsvoorbeeld kon niet worden voorbereid.",
        (Idioma::NoruegoNynorsk, "plan_failed") => "Forhåndsvisningen av endringen kunne ikke forberedes.",
        (Idioma::Polaco, "plan_failed") => "Nie można przygotować podglądu zmian.",
        (Idioma::PortuguesBrasil, "plan_failed") => "A visualização da mudança não pôde ser preparada.",
        (Idioma::Ruso, "plan_failed") => "Не удалось подготовить предварительный просмотр изменений.",
        (Idioma::Sueco, "plan_failed") => "Ändringsförhandsgranskningen kunde inte förberedas.",
        (Idioma::Turco, "plan_failed") => "The change preview could not be prepared.",
        (Idioma::Ucraniano, "plan_failed") => "Не вдалося підготувати попередній перегляд змін.",
        (Idioma::Vietnamita, "plan_failed") => "Không thể chuẩn bị bản xem trước thay đổi.",
        (Idioma::ChinoSimplificado, "plan_failed") => "无法准备更改预览。",
        (Idioma::Ingles, "sections") => "Sections",
        (Idioma::BelarusLatino, "sections") => "Razdzjeły",
        (Idioma::Belarus, "sections") => "Раздзелы",
        (Idioma::Catalan, "sections") => "Seccions",
        (Idioma::Checo, "sections") => "Sekce",
        (Idioma::Aleman, "sections") => "Abschnitte",
        (Idioma::Frances, "sections") => "Sections",
        (Idioma::Gallego, "sections") => "Seccións",
        (Idioma::Italiano, "sections") => "Sezioni",
        (Idioma::Coreano, "sections") => "섹션",
        (Idioma::Kurdo, "sections") => "Beşên",
        (Idioma::Neerlandes, "sections") => "Secties",
        (Idioma::NoruegoNynorsk, "sections") => "Seksjoner",
        (Idioma::Polaco, "sections") => "Sekcje",
        (Idioma::PortuguesBrasil, "sections") => "Seções",
        (Idioma::Ruso, "sections") => "Разделы",
        (Idioma::Sueco, "sections") => "Avsnitt",
        (Idioma::Turco, "sections") => "Bölümler",
        (Idioma::Ucraniano, "sections") => "Розділи",
        (Idioma::Vietnamita, "sections") => "Phần",
        (Idioma::ChinoSimplificado, "sections") => "部分",
        (Idioma::Ingles, "media") => "Sound and camera",
        (Idioma::BelarusLatino, "media") => "Huk i kamjera",
        (Idioma::Belarus, "media") => "Гук і камера",
        (Idioma::Catalan, "media") => "So i càmera",
        (Idioma::Checo, "media") => "Zvuk a kamera",
        (Idioma::Aleman, "media") => "Ton und Kamera",
        (Idioma::Frances, "media") => "Son et caméra",
        (Idioma::Gallego, "media") => "Son e cámara",
        (Idioma::Italiano, "media") => "Suono e fotocamera",
        (Idioma::Coreano, "media") => "사운드 및 카메라",
        (Idioma::Kurdo, "media") => "Deng û kamera",
        (Idioma::Neerlandes, "media") => "Geluid en camera",
        (Idioma::NoruegoNynorsk, "media") => "Lyd og kamera",
        (Idioma::Polaco, "media") => "Dźwięk i kamera",
        (Idioma::PortuguesBrasil, "media") => "Som e câmera",
        (Idioma::Ruso, "media") => "Звук и камера",
        (Idioma::Sueco, "media") => "Ljud och kamera",
        (Idioma::Turco, "media") => "Ses ve kamera",
        (Idioma::Ucraniano, "media") => "Звук і камера",
        (Idioma::Vietnamita, "media") => "Âm thanh và camera",
        (Idioma::ChinoSimplificado, "media") => "声音和摄像头",
        (Idioma::Ingles, "storage") => "Storage",
        (Idioma::BelarusLatino, "storage") => "Zachoŭvannje",
        (Idioma::Belarus, "storage") => "Захоўванне",
        (Idioma::Catalan, "storage") => "Emmagatzematge",
        (Idioma::Checo, "storage") => "Úložiště",
        (Idioma::Aleman, "storage") => "Lagerung",
        (Idioma::Frances, "storage") => "Stockage",
        (Idioma::Gallego, "storage") => "Almacenamento",
        (Idioma::Italiano, "storage") => "Deposito",
        (Idioma::Coreano, "storage") => "스토리지",
        (Idioma::Kurdo, "storage") => "Storage",
        (Idioma::Neerlandes, "storage") => "Opslag",
        (Idioma::NoruegoNynorsk, "storage") => "Lagring",
        (Idioma::Polaco, "storage") => "Przechowywanie",
        (Idioma::PortuguesBrasil, "storage") => "Armazenamento",
        (Idioma::Ruso, "storage") => "Хранение",
        (Idioma::Sueco, "storage") => "Lagring",
        (Idioma::Turco, "storage") => "Depolama",
        (Idioma::Ucraniano, "storage") => "Зберігання",
        (Idioma::Vietnamita, "storage") => "Lưu trữ",
        (Idioma::ChinoSimplificado, "storage") => "存储",
        (Idioma::Ingles, "firmware_updates") => "Firmware",
        (Idioma::BelarusLatino, "firmware_updates") => "Prašyŭka",
        (Idioma::Belarus, "firmware_updates") => "Прашыўка",
        (Idioma::Catalan, "firmware_updates") => "Firmware",
        (Idioma::Checo, "firmware_updates") => "Firmware",
        (Idioma::Aleman, "firmware_updates") => "Firmware",
        (Idioma::Frances, "firmware_updates") => "Micrologiciel",
        (Idioma::Gallego, "firmware_updates") => "Firmware",
        (Idioma::Italiano, "firmware_updates") => "Firmware",
        (Idioma::Coreano, "firmware_updates") => "펌웨어",
        (Idioma::Kurdo, "firmware_updates") => "Firmware",
        (Idioma::Neerlandes, "firmware_updates") => "Firmware",
        (Idioma::NoruegoNynorsk, "firmware_updates") => "Fastvare",
        (Idioma::Polaco, "firmware_updates") => "Oprogramowanie sprzętowe",
        (Idioma::PortuguesBrasil, "firmware_updates") => "Firmware",
        (Idioma::Ruso, "firmware_updates") => "Прошивка",
        (Idioma::Sueco, "firmware_updates") => "Firmware",
        (Idioma::Turco, "firmware_updates") => "Firmware",
        (Idioma::Ucraniano, "firmware_updates") => "Прошивка",
        (Idioma::Vietnamita, "firmware_updates") => "Phần sụn",
        (Idioma::ChinoSimplificado, "firmware_updates") => "固件",
        (Idioma::Ingles, "maintenance") => "Maintenance",
        (Idioma::BelarusLatino, "maintenance") => "Techničnaje absłuhoŭvannje",
        (Idioma::Belarus, "maintenance") => "Тэхнічнае абслугоўванне",
        (Idioma::Catalan, "maintenance") => "Manteniment",
        (Idioma::Checo, "maintenance") => "Údržba",
        (Idioma::Aleman, "maintenance") => "Wartung",
        (Idioma::Frances, "maintenance") => "Entretien",
        (Idioma::Gallego, "maintenance") => "Mantemento\nModelo __KX00064__",
        (Idioma::Italiano, "maintenance") => "Manutenzione",
        (Idioma::Coreano, "maintenance") => "유지 관리",
        (Idioma::Kurdo, "maintenance") => "Maintenance",
        (Idioma::Neerlandes, "maintenance") => "Onderhoud",
        (Idioma::NoruegoNynorsk, "maintenance") => "Vedlikehold",
        (Idioma::Polaco, "maintenance") => "Konserwacja",
        (Idioma::PortuguesBrasil, "maintenance") => "Manutenção",
        (Idioma::Ruso, "maintenance") => "Техническое обслуживание",
        (Idioma::Sueco, "maintenance") => "Underhåll",
        (Idioma::Turco, "maintenance") => "Bakım",
        (Idioma::Ucraniano, "maintenance") => "Технічне обслуговування",
        (Idioma::Vietnamita, "maintenance") => "Bảo trì",
        (Idioma::ChinoSimplificado, "maintenance") => "维护",
        (Idioma::Ingles, "recovery") => "Recovery versions",
        (Idioma::BelarusLatino, "recovery") => "Vjersii dłja adnaŭłjennja",
        (Idioma::Belarus, "recovery") => "Версіі для аднаўлення",
        (Idioma::Catalan, "recovery") => "Versions de recuperació",
        (Idioma::Checo, "recovery") => "Verze pro obnovení",
        (Idioma::Aleman, "recovery") => "Wiederherstellungsversionen",
        (Idioma::Frances, "recovery") => "Versions de récupération",
        (Idioma::Gallego, "recovery") => "Versións de recuperación",
        (Idioma::Italiano, "recovery") => "Versioni di ripristino",
        (Idioma::Coreano, "recovery") => "복구 버전",
        (Idioma::Kurdo, "recovery") => "Guhertoyên hilanînê",
        (Idioma::Neerlandes, "recovery") => "Herstelversies",
        (Idioma::NoruegoNynorsk, "recovery") => "Gjenopprettingsversjoner",
        (Idioma::Polaco, "recovery") => "Wersje odzyskiwania",
        (Idioma::PortuguesBrasil, "recovery") => "Versões de recuperação",
        (Idioma::Ruso, "recovery") => "Версии восстановления",
        (Idioma::Sueco, "recovery") => "Återställningsversioner",
        (Idioma::Turco, "recovery") => "Kurtarma sürümleri",
        (Idioma::Ucraniano, "recovery") => "Версії для відновлення",
        (Idioma::Vietnamita, "recovery") => "Phiên bản khôi phục",
        (Idioma::ChinoSimplificado, "recovery") => "恢复版本",
        (Idioma::Ingles, "cleanup") => "Cleanup",
        (Idioma::BelarusLatino, "cleanup") => "Ačystka",
        (Idioma::Belarus, "cleanup") => "Ачыстка",
        (Idioma::Catalan, "cleanup") => "Neteja",
        (Idioma::Checo, "cleanup") => "Vyčištění",
        (Idioma::Aleman, "cleanup") => "Bereinigung",
        (Idioma::Frances, "cleanup") => "Nettoyage",
        (Idioma::Gallego, "cleanup") => "Limpeza",
        (Idioma::Italiano, "cleanup") => "Pulizia",
        (Idioma::Coreano, "cleanup") => "정리",
        (Idioma::Kurdo, "cleanup") => "Paqijkirin",
        (Idioma::Neerlandes, "cleanup") => "Opruimen",
        (Idioma::NoruegoNynorsk, "cleanup") => "Opprydding",
        (Idioma::Polaco, "cleanup") => "Czyszczenie",
        (Idioma::PortuguesBrasil, "cleanup") => "Limpeza",
        (Idioma::Ruso, "cleanup") => "Очистка",
        (Idioma::Sueco, "cleanup") => "Rensning",
        (Idioma::Turco, "cleanup") => "Temizleme",
        (Idioma::Ucraniano, "cleanup") => "Очищення",
        (Idioma::Vietnamita, "cleanup") => "Dọn dẹp",
        (Idioma::ChinoSimplificado, "cleanup") => "清理",
        (Idioma::Ingles, "normal_cleanup") => "Recommended cleanup",
        (Idioma::BelarusLatino, "normal_cleanup") => "Rekamjendavanaja ačystka",
        (Idioma::Belarus, "normal_cleanup") => "Рэкамендаваная ачыстка",
        (Idioma::Catalan, "normal_cleanup") => "Neteja recomanada",
        (Idioma::Checo, "normal_cleanup") => "Doporučené čištění",
        (Idioma::Aleman, "normal_cleanup") => "Empfohlene Bereinigung",
        (Idioma::Frances, "normal_cleanup") => "Nettoyage recommandé",
        (Idioma::Gallego, "normal_cleanup") => "Limpeza recomendada",
        (Idioma::Italiano, "normal_cleanup") => "Pulizia consigliata",
        (Idioma::Coreano, "normal_cleanup") => "권장 정리",
        (Idioma::Kurdo, "normal_cleanup") => "Paqijkirina pêşniyar kirin",
        (Idioma::Neerlandes, "normal_cleanup") => "Aanbevolen schoonmaakbeurt",
        (Idioma::NoruegoNynorsk, "normal_cleanup") => "Anbefalt opprydding",
        (Idioma::Polaco, "normal_cleanup") => "Zalecane czyszczenie",
        (Idioma::PortuguesBrasil, "normal_cleanup") => "Limpeza recomendada",
        (Idioma::Ruso, "normal_cleanup") => "Рекомендуемая очистка",
        (Idioma::Sueco, "normal_cleanup") => "Rekommenderad rengöring",
        (Idioma::Turco, "normal_cleanup") => "Önerilen temizleme",
        (Idioma::Ucraniano, "normal_cleanup") => "Рекомендоване очищення",
        (Idioma::Vietnamita, "normal_cleanup") => "Đề xuất dọn dẹp",
        (Idioma::ChinoSimplificado, "normal_cleanup") => "建议清理",
        (Idioma::Ingles, "deep_cleanup") => "Deep cleanup",
        (Idioma::BelarusLatino, "deep_cleanup") => "Hłybokaja ačystka",
        (Idioma::Belarus, "deep_cleanup") => "Глыбокая ачыстка",
        (Idioma::Catalan, "deep_cleanup") => "Neteja profunda",
        (Idioma::Checo, "deep_cleanup") => "Hluboké čištění",
        (Idioma::Aleman, "deep_cleanup") => "Gründliche Bereinigung",
        (Idioma::Frances, "deep_cleanup") => "Nettoyage en profondeur",
        (Idioma::Gallego, "deep_cleanup") => "Limpeza profunda",
        (Idioma::Italiano, "deep_cleanup") => "Pulizia profonda",
        (Idioma::Coreano, "deep_cleanup") => "심층 정리",
        (Idioma::Kurdo, "deep_cleanup") => "Paqijkirina kûr",
        (Idioma::Neerlandes, "deep_cleanup") => "Diepgaande schoonmaak",
        (Idioma::NoruegoNynorsk, "deep_cleanup") => "Dyp opprydding",
        (Idioma::Polaco, "deep_cleanup") => "Dokładne sprzątanie",
        (Idioma::PortuguesBrasil, "deep_cleanup") => "Limpeza profunda",
        (Idioma::Ruso, "deep_cleanup") => "Глубокая очистка",
        (Idioma::Sueco, "deep_cleanup") => "Djup rengöring",
        (Idioma::Turco, "deep_cleanup") => "Derinlemesine temizlik",
        (Idioma::Ucraniano, "deep_cleanup") => "Глибоке очищення",
        (Idioma::Vietnamita, "deep_cleanup") => "Dọn dẹp sâu",
        (Idioma::ChinoSimplificado, "deep_cleanup") => "深度清理",
        (Idioma::Ingles, "clean_now") => "Clean",
        (Idioma::BelarusLatino, "clean_now") => "Čystaja",
        (Idioma::Belarus, "clean_now") => "Чысты",
        (Idioma::Catalan, "clean_now") => "Neteja",
        (Idioma::Checo, "clean_now") => "Čistý",
        (Idioma::Aleman, "clean_now") => "Sauber",
        (Idioma::Frances, "clean_now") => "Nettoyer",
        (Idioma::Gallego, "clean_now") => "Limpar",
        (Idioma::Italiano, "clean_now") => "Pulito",
        (Idioma::Coreano, "clean_now") => "청소",
        (Idioma::Kurdo, "clean_now") => "Paqij",
        (Idioma::Neerlandes, "clean_now") => "Schoon",
        (Idioma::NoruegoNynorsk, "clean_now") => "Rengjør",
        (Idioma::Polaco, "clean_now") => "Czyste",
        (Idioma::PortuguesBrasil, "clean_now") => "Limpar",
        (Idioma::Ruso, "clean_now") => "Очистить",
        (Idioma::Sueco, "clean_now") => "Rengör",
        (Idioma::Turco, "clean_now") => "Temiz",
        (Idioma::Ucraniano, "clean_now") => "Чистий",
        (Idioma::Vietnamita, "clean_now") => "Sạch sẽ",
        (Idioma::ChinoSimplificado, "clean_now") => "清洁",
        (Idioma::Ingles, "clean_all") => "Clean more",
        (Idioma::BelarusLatino, "clean_all") => "Bołš čyscicje",
        (Idioma::Belarus, "clean_all") => "Больш чысціце",
        (Idioma::Catalan, "clean_all") => "Neteja més",
        (Idioma::Checo, "clean_all") => "Čistěte více",
        (Idioma::Aleman, "clean_all") => "Mehr reinigen",
        (Idioma::Frances, "clean_all") => "Nettoyer davantage",
        (Idioma::Gallego, "clean_all") => "Limpar máis",
        (Idioma::Italiano, "clean_all") => "Pulisci di più",
        (Idioma::Coreano, "clean_all") => "더 깨끗하게 청소하세요",
        (Idioma::Kurdo, "clean_all") => "Zêdetir paqij bike",
        (Idioma::Neerlandes, "clean_all") => "Maak meer schoon",
        (Idioma::NoruegoNynorsk, "clean_all") => "Rengjør mer",
        (Idioma::Polaco, "clean_all") => "Wyczyść więcej",
        (Idioma::PortuguesBrasil, "clean_all") => "Limpe mais",
        (Idioma::Ruso, "clean_all") => "Очистите больше",
        (Idioma::Sueco, "clean_all") => "Rengör mer",
        (Idioma::Turco, "clean_all") => "Daha fazlasını temizle",
        (Idioma::Ucraniano, "clean_all") => "Чистіть більше",
        (Idioma::Vietnamita, "clean_all") => "Dọn dẹp nhiều hơn",
        (Idioma::ChinoSimplificado, "clean_all") => "清洁更多",
        (Idioma::Ingles, "generations") => "Available versions",
        (Idioma::BelarusLatino, "generations") => "Dastupnyja vjersii",
        (Idioma::Belarus, "generations") => "Даступныя версіі",
        (Idioma::Catalan, "generations") => "Versions disponibles",
        (Idioma::Checo, "generations") => "Dostupné verze",
        (Idioma::Aleman, "generations") => "Verfügbare Versionen",
        (Idioma::Frances, "generations") => "Versions disponibles",
        (Idioma::Gallego, "generations") => "Versións dispoñibles",
        (Idioma::Italiano, "generations") => "Versioni disponibili",
        (Idioma::Coreano, "generations") => "사용 가능한 버전",
        (Idioma::Kurdo, "generations") => "Guhertoyên berdest",
        (Idioma::Neerlandes, "generations") => "Beschikbare versies",
        (Idioma::NoruegoNynorsk, "generations") => "Tilgjengelige versjoner",
        (Idioma::Polaco, "generations") => "Dostępne wersje",
        (Idioma::PortuguesBrasil, "generations") => "Versões disponíveis",
        (Idioma::Ruso, "generations") => "Доступные версии",
        (Idioma::Sueco, "generations") => "Tillgängliga versioner",
        (Idioma::Turco, "generations") => "Available versions",
        (Idioma::Ucraniano, "generations") => "Доступні версії",
        (Idioma::Vietnamita, "generations") => "Phiên bản có sẵn",
        (Idioma::ChinoSimplificado, "generations") => "可用版本",
        (Idioma::Ingles, "use_once") => "Try on next restart",
        (Idioma::BelarusLatino, "use_once") => "Pasprabujcje nastupny pjerazapusk",
        (Idioma::Belarus, "use_once") => "Паспрабуйце пры наступным перазапуску",
        (Idioma::Catalan, "use_once") => "Prova el proper reinici",
        (Idioma::Checo, "use_once") => "Zkuste další restart",
        (Idioma::Aleman, "use_once") => "Versuchen Sie es beim nächsten Neustart",
        (Idioma::Frances, "use_once") => "Essayez au prochain redémarrage",
        (Idioma::Gallego, "use_once") => "Proba o próximo reinicio",
        (Idioma::Italiano, "use_once") => "Prova al prossimo riavvio",
        (Idioma::Coreano, "use_once") => "다음에 다시 시작해 보세요.",
        (Idioma::Kurdo, "use_once") => "Ji nû ve destpêkirina din biceribîne",
        (Idioma::Neerlandes, "use_once") => "Probeer bij de volgende herstart",
        (Idioma::NoruegoNynorsk, "use_once") => "Prøv neste omstart",
        (Idioma::Polaco, "use_once") => "Spróbuj przy następnym uruchomieniu",
        (Idioma::PortuguesBrasil, "use_once") => "Tente na próxima reinicialização",
        (Idioma::Ruso, "use_once") => "Попробуйте при следующем перезапуске",
        (Idioma::Sueco, "use_once") => "Prova nästa omstart",
        (Idioma::Turco, "use_once") => "Bir sonraki yeniden başlatmada deneyin",
        (Idioma::Ucraniano, "use_once") => "Спробуйте наступного перезапуску",
        (Idioma::Vietnamita, "use_once") => "Hãy thử khởi động lại lần sau",
        (Idioma::ChinoSimplificado, "use_once") => "尝试下次重新启动",
        (Idioma::Ingles, "current_generation") => "Version in use",
        (Idioma::BelarusLatino, "current_generation") => "Vjersija, jakaja vykarystoŭvajecca",
        (Idioma::Belarus, "current_generation") => "Версія, якая выкарыстоўваецца",
        (Idioma::Catalan, "current_generation") => "Versió en ús",
        (Idioma::Checo, "current_generation") => "Verze se používá",
        (Idioma::Aleman, "current_generation") => "Version wird verwendet",
        (Idioma::Frances, "current_generation") => "Version utilisée",
        (Idioma::Gallego, "current_generation") => "Versión en uso",
        (Idioma::Italiano, "current_generation") => "Versione in uso",
        (Idioma::Coreano, "current_generation") => "사용 중인 버전",
        (Idioma::Kurdo, "current_generation") => "Guhertoya ku tê bikaranîn",
        (Idioma::Neerlandes, "current_generation") => "Versie in gebruik",
        (Idioma::NoruegoNynorsk, "current_generation") => "Versjon i bruk",
        (Idioma::Polaco, "current_generation") => "Wersja w użyciu",
        (Idioma::PortuguesBrasil, "current_generation") => "Versão em uso",
        (Idioma::Ruso, "current_generation") => "Используемая версия",
        (Idioma::Sueco, "current_generation") => "Version som används",
        (Idioma::Turco, "current_generation") => "Sürüm kullanımda",
        (Idioma::Ucraniano, "current_generation") => "Версія, що використовується",
        (Idioma::Vietnamita, "current_generation") => "Phiên bản đang được sử dụng",
        (Idioma::ChinoSimplificado, "current_generation") => "使用中的版本",
        (Idioma::Ingles, "default_generation") => "Version used at startup",
        (Idioma::BelarusLatino, "default_generation") => "Vjersija, jakaja vykarystoŭvajecca pry zapusku",
        (Idioma::Belarus, "default_generation") => "Версія, якая выкарыстоўваецца пры запуску",
        (Idioma::Catalan, "default_generation") => "Versió utilitzada a l'inici",
        (Idioma::Checo, "default_generation") => "Verze použitá při spuštění",
        (Idioma::Aleman, "default_generation") => "Beim Start verwendete Version",
        (Idioma::Frances, "default_generation") => "Version utilisée au démarrage",
        (Idioma::Gallego, "default_generation") => "Versión utilizada no inicio",
        (Idioma::Italiano, "default_generation") => "Versione utilizzata all'avvio",
        (Idioma::Coreano, "default_generation") => "시작 시 사용되는 버전",
        (Idioma::Kurdo, "default_generation") => "Guhertoya ku di destpêkê de hatî bikar anîn",
        (Idioma::Neerlandes, "default_generation") => "Versie gebruikt bij het opstarten",
        (Idioma::NoruegoNynorsk, "default_generation") => "Versjon brukt ved oppstart",
        (Idioma::Polaco, "default_generation") => "Wersja używana podczas uruchamiania",
        (Idioma::PortuguesBrasil, "default_generation") => "Versão usada na inicialização",
        (Idioma::Ruso, "default_generation") => "Версия, используемая при запуске",
        (Idioma::Sueco, "default_generation") => "Version som användes vid start",
        (Idioma::Turco, "default_generation") => "Başlangıçta kullanılan sürüm",
        (Idioma::Ucraniano, "default_generation") => "Версія, яка використовується під час запуску",
        (Idioma::Vietnamita, "default_generation") => "Phiên bản được sử dụng khi khởi động",
        (Idioma::ChinoSimplificado, "default_generation") => "启动时使用的版本",
        (Idioma::Ingles, "heavy_transfer") => "Wait until data finishes saving",
        (Idioma::BelarusLatino, "heavy_transfer") => "Dačakajcjesja zakančennja zachavannja danych",
        (Idioma::Belarus, "heavy_transfer") => "Дачакайцеся завяршэння захавання даных",
        (Idioma::Catalan, "heavy_transfer") => "Espereu fins que les dades s'acabin de desar",
        (Idioma::Checo, "heavy_transfer") => "Počkejte na dokončení ukládání dat",
        (Idioma::Aleman, "heavy_transfer") => "Warten Sie, bis die Datenspeicherung abgeschlossen ist",
        (Idioma::Frances, "heavy_transfer") => "Attendez la fin de l'enregistrement des données",
        (Idioma::Gallego, "heavy_transfer") => "Agarde ata que se rematen de gardar os datos",
        (Idioma::Italiano, "heavy_transfer") => "Attendere il completamento del salvataggio dei dati",
        (Idioma::Coreano, "heavy_transfer") => "데이터 저장이 완료될 때까지 기다리세요",
        (Idioma::Kurdo, "heavy_transfer") => "Li bendê bin heya tomarkirina daneyan biqede",
        (Idioma::Neerlandes, "heavy_transfer") => "Wacht totdat de gegevens zijn opgeslagen",
        (Idioma::NoruegoNynorsk, "heavy_transfer") => "Vent til data er ferdig lagret",
        (Idioma::Polaco, "heavy_transfer") => "Poczekaj, aż zapisywanie danych zakończy się",
        (Idioma::PortuguesBrasil, "heavy_transfer") => "Aguarde até que os dados terminem de ser salvos",
        (Idioma::Ruso, "heavy_transfer") => "Подождите, пока данные не закончат сохраняться.",
        (Idioma::Sueco, "heavy_transfer") => "Vänta tills data har sparats",
        (Idioma::Turco, "heavy_transfer") => "Verilerin kaydedilmesi bitene kadar bekleyin",
        (Idioma::Ucraniano, "heavy_transfer") => "Зачекайте, доки завершиться збереження даних",
        (Idioma::Vietnamita, "heavy_transfer") => "Chờ cho đến khi lưu dữ liệu xong",
        (Idioma::ChinoSimplificado, "heavy_transfer") => "等待数据保存完成",
        (Idioma::Ingles, "heavy_transfer_detail") => {
            "Before ejecting, wait until all data has finished saving to the drive."
        }
        (Idioma::BelarusLatino, "heavy_transfer_detail") => {
            "Pjerš čym vynjac, pačakajcje, pakuł usje dadzjenyja nje buduc zachavany na dysk."
        }
        (Idioma::Belarus, "heavy_transfer_detail") => {
            "Перш чым выняць, пачакайце, пакуль усе дадзеныя не будуць захаваны на дыск."
        }
        (Idioma::Catalan, "heavy_transfer_detail") => {
            "Abans d'expulsar, espereu fins que totes les dades s'hagin desat a la unitat."
        }
        (Idioma::Checo, "heavy_transfer_detail") => {
            "Před vysunutím počkejte, dokud se všechna data neuloží na disk."
        }
        (Idioma::Aleman, "heavy_transfer_detail") => {
            "Warten Sie vor dem Auswerfen, bis alle Daten auf dem Laufwerk gespeichert wurden."
        }
        (Idioma::Frances, "heavy_transfer_detail") => {
            "Avant d'éjecter, attendez que toutes les données soient enregistrées sur le disque."
        }
        (Idioma::Gallego, "heavy_transfer_detail") => {
            "Antes de expulsar, agarde ata que rematen de gardar todos os datos na unidade."
        }
        (Idioma::Italiano, "heavy_transfer_detail") => {
            "Prima dell'espulsione, attendere il completamento del salvataggio di tutti i dati sull'unità."
        }
        (Idioma::Coreano, "heavy_transfer_detail") => {
            "꺼내기 전에 모든 데이터가 드라이브에 저장이 완료될 때까지 기다리십시오."
        }
        (Idioma::Kurdo, "heavy_transfer_detail") => {
            "Berî derxistinê, li bendê bimînin heya ku hemî dane tomarkirina li ajokê biqede."
        }
        (Idioma::Neerlandes, "heavy_transfer_detail") => {
            "Wacht vóór het uitwerpen tot alle gegevens op de schijf zijn opgeslagen."
        }
        (Idioma::NoruegoNynorsk, "heavy_transfer_detail") => {
            "Før du løser ut, vent til alle data er ferdig lagret på stasjonen."
        }
        (Idioma::Polaco, "heavy_transfer_detail") => {
            "Przed wysunięciem poczekaj, aż wszystkie dane zostaną zapisane na dysku."
        }
        (Idioma::PortuguesBrasil, "heavy_transfer_detail") => {
            "Antes de ejetar, espere até que todos os dados sejam salvos na unidade."
        }
        (Idioma::Ruso, "heavy_transfer_detail") => {
            "Перед извлечением подождите, пока все данные не будут сохранены на диске."
        }
        (Idioma::Sueco, "heavy_transfer_detail") => {
            "Innan du matar ut, vänta tills all data har sparats på enheten."
        }
        (Idioma::Turco, "heavy_transfer_detail") => {
            "Before ejecting, wait until all data has finished saving to the drive."
        }
        (Idioma::Ucraniano, "heavy_transfer_detail") => {
            "Перед вийманням зачекайте, доки всі дані не будуть збережені на диску."
        }
        (Idioma::Vietnamita, "heavy_transfer_detail") => {
            "Trước khi đẩy ra, hãy đợi cho đến khi tất cả dữ liệu được lưu vào ổ đĩa xong."
        }
        (Idioma::ChinoSimplificado, "heavy_transfer_detail") => {
            "在弹出之前，请等待所有数据完成保存到驱动器。"
        }
        (Idioma::Ingles, "eject") => "Eject",
        (Idioma::BelarusLatino, "eject") => "Vynjac",
        (Idioma::Belarus, "eject") => "Выняць",
        (Idioma::Catalan, "eject") => "Expulsar",
        (Idioma::Checo, "eject") => "Vysunout",
        (Idioma::Aleman, "eject") => "Auswerfen",
        (Idioma::Frances, "eject") => "Éjecter",
        (Idioma::Gallego, "eject") => "Expulsar",
        (Idioma::Italiano, "eject") => "Espulsione",
        (Idioma::Coreano, "eject") => "꺼내기",
        (Idioma::Kurdo, "eject") => "Derxistin",
        (Idioma::Neerlandes, "eject") => "Uitwerpen",
        (Idioma::NoruegoNynorsk, "eject") => "Løs ut",
        (Idioma::Polaco, "eject") => "Wysuń",
        (Idioma::PortuguesBrasil, "eject") => "Ejetar",
        (Idioma::Ruso, "eject") => "Извлечь",
        (Idioma::Sueco, "eject") => "Mata ut",
        (Idioma::Turco, "eject") => "Çıkar",
        (Idioma::Ucraniano, "eject") => "Витягнути",
        (Idioma::Vietnamita, "eject") => "Đẩy ra",
        (Idioma::ChinoSimplificado, "eject") => "弹出",
        (Idioma::Ingles, "removable") => "Removable",
        (Idioma::BelarusLatino, "removable") => "Zdymnaja",
        (Idioma::Belarus, "removable") => "Здымны",
        (Idioma::Catalan, "removable") => "Extraïble",
        (Idioma::Checo, "removable") => "Odnímatelné",
        (Idioma::Aleman, "removable") => "Abnehmbar",
        (Idioma::Frances, "removable") => "Amovible",
        (Idioma::Gallego, "removable") => "Extraíble",
        (Idioma::Italiano, "removable") => "Rimovibile",
        (Idioma::Coreano, "removable") => "이동식",
        (Idioma::Kurdo, "removable") => "Rakirin",
        (Idioma::Neerlandes, "removable") => "Verwijderbaar",
        (Idioma::NoruegoNynorsk, "removable") => "Avtakbar",
        (Idioma::Polaco, "removable") => "Wyjmowany",
        (Idioma::PortuguesBrasil, "removable") => "Removível",
        (Idioma::Ruso, "removable") => "Съемный",
        (Idioma::Sueco, "removable") => "Avtagbar",
        (Idioma::Turco, "removable") => "Çıkarılabilir",
        (Idioma::Ucraniano, "removable") => "Знімний",
        (Idioma::Vietnamita, "removable") => "Có thể tháo rời",
        (Idioma::ChinoSimplificado, "removable") => "可拆卸",
        (Idioma::Ingles, "internal") => "Internal",
        (Idioma::BelarusLatino, "internal") => "Unutrany",
        (Idioma::Belarus, "internal") => "Унутраны",
        (Idioma::Catalan, "internal") => "Intern",
        (Idioma::Checo, "internal") => "Interní",
        (Idioma::Aleman, "internal") => "Intern",
        (Idioma::Frances, "internal") => "Interne",
        (Idioma::Gallego, "internal") => "Interna",
        (Idioma::Italiano, "internal") => "Interno",
        (Idioma::Coreano, "internal") => "내부",
        (Idioma::Kurdo, "internal") => "Navxweyî",
        (Idioma::Neerlandes, "internal") => "Intern",
        (Idioma::NoruegoNynorsk, "internal") => "Intern",
        (Idioma::Polaco, "internal") => "Wewnętrzne",
        (Idioma::PortuguesBrasil, "internal") => "Interno",
        (Idioma::Ruso, "internal") => "Внутренний",
        (Idioma::Sueco, "internal") => "Internt",
        (Idioma::Turco, "internal") => "Dahili",
        (Idioma::Ucraniano, "internal") => "Внутрішній",
        (Idioma::Vietnamita, "internal") => "Nội bộ",
        (Idioma::ChinoSimplificado, "internal") => "内部",
        (Idioma::Ingles, "safe_disconnect") => "Safe to disconnect.",
        (Idioma::BelarusLatino, "safe_disconnect") => "Bjaspječna adkłjučacca.",
        (Idioma::Belarus, "safe_disconnect") => "Бяспечна адключыць.",
        (Idioma::Catalan, "safe_disconnect") => "Segur per desconnectar.",
        (Idioma::Checo, "safe_disconnect") => "Bezpečné odpojení.",
        (Idioma::Aleman, "safe_disconnect") => "Sicheres Trennen.",
        (Idioma::Frances, "safe_disconnect") => "Déconnexion sûre.",
        (Idioma::Gallego, "safe_disconnect") => "Seguro para desconectar.",
        (Idioma::Italiano, "safe_disconnect") => "Disconnessione sicura.",
        (Idioma::Coreano, "safe_disconnect") => "연결을 끊어도 안전합니다.",
        (Idioma::Kurdo, "safe_disconnect") => "Ji bo veqetandinê ewle ye.",
        (Idioma::Neerlandes, "safe_disconnect") => "Veilig te ontkoppelen.",
        (Idioma::NoruegoNynorsk, "safe_disconnect") => "Trygt å koble fra.",
        (Idioma::Polaco, "safe_disconnect") => "Można bezpiecznie odłączyć.",
        (Idioma::PortuguesBrasil, "safe_disconnect") => "É seguro desconectar.",
        (Idioma::Ruso, "safe_disconnect") => "Безопасное отключение.",
        (Idioma::Sueco, "safe_disconnect") => "Säkert att koppla från.",
        (Idioma::Turco, "safe_disconnect") => "Bağlantının kesilmesi güvenlidir.",
        (Idioma::Ucraniano, "safe_disconnect") => "Можна безпечно від’єднати.",
        (Idioma::Vietnamita, "safe_disconnect") => "Ngắt kết nối an toàn.",
        (Idioma::ChinoSimplificado, "safe_disconnect") => "可以安全断开连接。",
        (Idioma::Ingles, "available_updates") => "Firmware updates",
        (Idioma::BelarusLatino, "available_updates") => "Abnaŭłjennja prašyŭki",
        (Idioma::Belarus, "available_updates") => "Абнаўленні прашыўкі",
        (Idioma::Catalan, "available_updates") => "Actualitzacions de firmware",
        (Idioma::Checo, "available_updates") => "Aktualizace firmwaru",
        (Idioma::Aleman, "available_updates") => "Firmware-Updates",
        (Idioma::Frances, "available_updates") => "Mises à jour du micrologiciel",
        (Idioma::Gallego, "available_updates") => "Actualizacións de firmware",
        (Idioma::Italiano, "available_updates") => "Aggiornamenti firmware",
        (Idioma::Coreano, "available_updates") => "펌웨어 업데이트",
        (Idioma::Kurdo, "available_updates") => "Nûvekirinên Firmware",
        (Idioma::Neerlandes, "available_updates") => "Firmware-updates",
        (Idioma::NoruegoNynorsk, "available_updates") => "Fastvareoppdateringer",
        (Idioma::Polaco, "available_updates") => "Aktualizacje oprogramowania sprzętowego",
        (Idioma::PortuguesBrasil, "available_updates") => "Atualizações de firmware",
        (Idioma::Ruso, "available_updates") => "Обновления прошивки",
        (Idioma::Sueco, "available_updates") => "Firmwareuppdateringar",
        (Idioma::Turco, "available_updates") => "Firmware güncellemeleri",
        (Idioma::Ucraniano, "available_updates") => "Оновлення мікропрограми",
        (Idioma::Vietnamita, "available_updates") => "Cập nhật chương trình cơ sở",
        (Idioma::ChinoSimplificado, "available_updates") => "固件更新",
        (Idioma::Ingles, "firmware_devices") => "Firmware devices",
        (Idioma::BelarusLatino, "firmware_devices") => "Prašyŭka pryład",
        (Idioma::Belarus, "firmware_devices") => "Прашыўка прылад",
        (Idioma::Catalan, "firmware_devices") => "Dispositius de microprogramari",
        (Idioma::Checo, "firmware_devices") => "Firmwarová zařízení",
        (Idioma::Aleman, "firmware_devices") => "Firmware-Geräte",
        (Idioma::Frances, "firmware_devices") => "Micrologiciel des appareils",
        (Idioma::Gallego, "firmware_devices") => "Dispositivos de firmware",
        (Idioma::Italiano, "firmware_devices") => "Dispositivi firmware",
        (Idioma::Coreano, "firmware_devices") => "펌웨어 장치",
        (Idioma::Kurdo, "firmware_devices") => "Amûrên Firmware",
        (Idioma::Neerlandes, "firmware_devices") => "Firmware-apparaten",
        (Idioma::NoruegoNynorsk, "firmware_devices") => "Fastvareenheter",
        (Idioma::Polaco, "firmware_devices") => "Urządzenia z oprogramowaniem sprzętowym",
        (Idioma::PortuguesBrasil, "firmware_devices") => "Dispositivos de firmware",
        (Idioma::Ruso, "firmware_devices") => "Прошивка устройства",
        (Idioma::Sueco, "firmware_devices") => "Firmware-enheter",
        (Idioma::Turco, "firmware_devices") => "Firmware devices",
        (Idioma::Ucraniano, "firmware_devices") => "Програмне забезпечення пристроїв",
        (Idioma::Vietnamita, "firmware_devices") => "Thiết bị phần mềm",
        (Idioma::ChinoSimplificado, "firmware_devices") => "固件设备",
        (Idioma::Ingles, "refresh_firmware") => "Check for updates",
        (Idioma::BelarusLatino, "refresh_firmware") => "Pravjercje najaŭnasc abnaŭłjennjaŭ",
        (Idioma::Belarus, "refresh_firmware") => "Праверце наяўнасць абнаўленняў",
        (Idioma::Catalan, "refresh_firmware") => "Comproveu si hi ha actualitzacions",
        (Idioma::Checo, "refresh_firmware") => "Zkontrolujte aktualizace",
        (Idioma::Aleman, "refresh_firmware") => "Nach Updates suchen",
        (Idioma::Frances, "refresh_firmware") => "Rechercher des mises à jour",
        (Idioma::Gallego, "refresh_firmware") => "Busca actualizacións",
        (Idioma::Italiano, "refresh_firmware") => "Controlla gli aggiornamenti",
        (Idioma::Coreano, "refresh_firmware") => "업데이트 확인",
        (Idioma::Kurdo, "refresh_firmware") => "Ji bo nûvekirinan kontrol bikin",
        (Idioma::Neerlandes, "refresh_firmware") => "Controleer op updates",
        (Idioma::NoruegoNynorsk, "refresh_firmware") => "Se etter oppdateringer",
        (Idioma::Polaco, "refresh_firmware") => "Sprawdź dostępność aktualizacji",
        (Idioma::PortuguesBrasil, "refresh_firmware") => "Verifique se há atualizações",
        (Idioma::Ruso, "refresh_firmware") => "Проверить наличие обновлений",
        (Idioma::Sueco, "refresh_firmware") => "Sök efter uppdateringar",
        (Idioma::Turco, "refresh_firmware") => "Check for updates",
        (Idioma::Ucraniano, "refresh_firmware") => "Перевірити наявність оновлень",
        (Idioma::Vietnamita, "refresh_firmware") => "Kiểm tra cập nhật",
        (Idioma::ChinoSimplificado, "refresh_firmware") => "检查更新",
        (Idioma::Ingles, "install") => "Install",
        (Idioma::BelarusLatino, "install") => "Ustałjavac",
        (Idioma::Belarus, "install") => "Усталяваць",
        (Idioma::Catalan, "install") => "Instal·la",
        (Idioma::Checo, "install") => "Instalovat",
        (Idioma::Aleman, "install") => "Installieren",
        (Idioma::Frances, "install") => "Installer",
        (Idioma::Gallego, "install") => "Instalar",
        (Idioma::Italiano, "install") => "Installa",
        (Idioma::Coreano, "install") => "설치",
        (Idioma::Kurdo, "install") => "Saz bike",
        (Idioma::Neerlandes, "install") => "Installeren",
        (Idioma::NoruegoNynorsk, "install") => "Installer",
        (Idioma::Polaco, "install") => "Zainstaluj",
        (Idioma::PortuguesBrasil, "install") => "Instalar",
        (Idioma::Ruso, "install") => "Установить",
        (Idioma::Sueco, "install") => "Installera",
        (Idioma::Turco, "install") => "Yükle",
        (Idioma::Ucraniano, "install") => "Встановити",
        (Idioma::Vietnamita, "install") => "Cài đặt",
        (Idioma::ChinoSimplificado, "install") => "安装",
        (Idioma::Ingles, "no_updates") => "No updates available.",
        (Idioma::BelarusLatino, "no_updates") => "Njama dastupnych abnaŭłjennjaŭ.",
        (Idioma::Belarus, "no_updates") => "Няма даступных абнаўленняў.",
        (Idioma::Catalan, "no_updates") => "No hi ha actualitzacions disponibles.",
        (Idioma::Checo, "no_updates") => "Nejsou k dispozici žádné aktualizace.",
        (Idioma::Aleman, "no_updates") => "Keine Updates verfügbar.",
        (Idioma::Frances, "no_updates") => "Aucune mise à jour disponible.",
        (Idioma::Gallego, "no_updates") => "Non hai actualizacións dispoñibles.",
        (Idioma::Italiano, "no_updates") => "Nessun aggiornamento disponibile.",
        (Idioma::Coreano, "no_updates") => "사용 가능한 업데이트가 없습니다.",
        (Idioma::Kurdo, "no_updates") => "Nûvekirin tune.",
        (Idioma::Neerlandes, "no_updates") => "Geen updates beschikbaar.",
        (Idioma::NoruegoNynorsk, "no_updates") => "Ingen oppdateringer tilgjengelig.",
        (Idioma::Polaco, "no_updates") => "Brak dostępnych aktualizacji.",
        (Idioma::PortuguesBrasil, "no_updates") => "Nenhuma atualização disponível.",
        (Idioma::Ruso, "no_updates") => "Обновлений нет.",
        (Idioma::Sueco, "no_updates") => "Inga uppdateringar tillgängliga.",
        (Idioma::Turco, "no_updates") => "Güncelleme yok.",
        (Idioma::Ucraniano, "no_updates") => "Немає оновлень.",
        (Idioma::Vietnamita, "no_updates") => "Không có bản cập nhật nào.",
        (Idioma::ChinoSimplificado, "no_updates") => "无可用更新。",
        (Idioma::Ingles, "version") => "Version",
        (Idioma::BelarusLatino, "version") => "Vjersija",
        (Idioma::Belarus, "version") => "Версія",
        (Idioma::Catalan, "version") => "Versió",
        (Idioma::Checo, "version") => "Verze",
        (Idioma::Aleman, "version") => "Version",
        (Idioma::Frances, "version") => "Version",
        (Idioma::Gallego, "version") => "Versión",
        (Idioma::Italiano, "version") => "Versione",
        (Idioma::Coreano, "version") => "버전",
        (Idioma::Kurdo, "version") => "Versiyon",
        (Idioma::Neerlandes, "version") => "Versie",
        (Idioma::NoruegoNynorsk, "version") => "Versjon",
        (Idioma::Polaco, "version") => "Wersja",
        (Idioma::PortuguesBrasil, "version") => "Versão",
        (Idioma::Ruso, "version") => "Версия",
        (Idioma::Sueco, "version") => "Version",
        (Idioma::Turco, "version") => "Sürüm",
        (Idioma::Ucraniano, "version") => "Версія",
        (Idioma::Vietnamita, "version") => "Phiên bản",
        (Idioma::ChinoSimplificado, "version") => "版本",
        (Idioma::Ingles, "output") => "Sound output",
        (Idioma::BelarusLatino, "output") => "Hukavy vychad",
        (Idioma::Belarus, "output") => "Гукавы выхад",
        (Idioma::Catalan, "output") => "Sortida de so",
        (Idioma::Checo, "output") => "Zvukový výstup",
        (Idioma::Aleman, "output") => "Tonausgabe",
        (Idioma::Frances, "output") => "Sortie sonore",
        (Idioma::Gallego, "output") => "Saída de son",
        (Idioma::Italiano, "output") => "Uscita audio",
        (Idioma::Coreano, "output") => "사운드 출력",
        (Idioma::Kurdo, "output") => "Derketina deng",
        (Idioma::Neerlandes, "output") => "Geluidsuitvoer",
        (Idioma::NoruegoNynorsk, "output") => "Lydutgang",
        (Idioma::Polaco, "output") => "Wyjście dźwięku",
        (Idioma::PortuguesBrasil, "output") => "Saída de som",
        (Idioma::Ruso, "output") => "Звуковой выход",
        (Idioma::Sueco, "output") => "Ljudutgång",
        (Idioma::Turco, "output") => "Ses çıkışı",
        (Idioma::Ucraniano, "output") => "Звуковий вихід",
        (Idioma::Vietnamita, "output") => "Âm thanh phát ra",
        (Idioma::ChinoSimplificado, "output") => "声音输出",
        (Idioma::Ingles, "input") => "Microphone",
        (Idioma::BelarusLatino, "input") => "Mikrafon",
        (Idioma::Belarus, "input") => "Мікрафон",
        (Idioma::Catalan, "input") => "Micròfon",
        (Idioma::Checo, "input") => "Mikrofon",
        (Idioma::Aleman, "input") => "Mikrofon",
        (Idioma::Frances, "input") => "Micro",
        (Idioma::Gallego, "input") => "Micrófono",
        (Idioma::Italiano, "input") => "Microfono",
        (Idioma::Coreano, "input") => "마이크",
        (Idioma::Kurdo, "input") => "Mîkrofon",
        (Idioma::Neerlandes, "input") => "Microfoon",
        (Idioma::NoruegoNynorsk, "input") => "Mikrofon",
        (Idioma::Polaco, "input") => "Mikrofon",
        (Idioma::PortuguesBrasil, "input") => "Microfone",
        (Idioma::Ruso, "input") => "Микрофон",
        (Idioma::Sueco, "input") => "Mikrofon",
        (Idioma::Turco, "input") => "Mikrofon",
        (Idioma::Ucraniano, "input") => "Мікрофон",
        (Idioma::Vietnamita, "input") => "Micrô",
        (Idioma::ChinoSimplificado, "input") => "麦克风",
        (Idioma::Ingles, "volume") => "Volume",
        (Idioma::BelarusLatino, "volume") => "Ab'jom",
        (Idioma::Belarus, "volume") => "Гучнасць",
        (Idioma::Catalan, "volume") => "Volum",
        (Idioma::Checo, "volume") => "Hlasitost",
        (Idioma::Aleman, "volume") => "Lautstärke",
        (Idioma::Frances, "volume") => "Volume",
        (Idioma::Gallego, "volume") => "Volume",
        (Idioma::Italiano, "volume") => "Volume",
        (Idioma::Coreano, "volume") => "볼륨",
        (Idioma::Kurdo, "volume") => "Bilindahiya dengê",
        (Idioma::Neerlandes, "volume") => "Volume",
        (Idioma::NoruegoNynorsk, "volume") => "Ljodstyrke",
        (Idioma::Polaco, "volume") => "Głośność",
        (Idioma::PortuguesBrasil, "volume") => "Volume",
        (Idioma::Ruso, "volume") => "Громкость",
        (Idioma::Sueco, "volume") => "Volym",
        (Idioma::Turco, "volume") => "Ses",
        (Idioma::Ucraniano, "volume") => "Гучність",
        (Idioma::Vietnamita, "volume") => "Âm lượng",
        (Idioma::ChinoSimplificado, "volume") => "音量",
        (Idioma::Ingles, "mute") => "Mute",
        (Idioma::BelarusLatino, "mute") => "Adkłjučyc huk",
        (Idioma::Belarus, "mute") => "Адключыць гук",
        (Idioma::Catalan, "mute") => "Silenciar",
        (Idioma::Checo, "mute") => "Ztlumit",
        (Idioma::Aleman, "mute") => "Stumm",
        (Idioma::Frances, "mute") => "Muet",
        (Idioma::Gallego, "mute") => "Silenciar",
        (Idioma::Italiano, "mute") => "Muto",
        (Idioma::Coreano, "mute") => "음소거",
        (Idioma::Kurdo, "mute") => "Bêdeng bike",
        (Idioma::Neerlandes, "mute") => "Dempen",
        (Idioma::NoruegoNynorsk, "mute") => "Demp",
        (Idioma::Polaco, "mute") => "Wycisz",
        (Idioma::PortuguesBrasil, "mute") => "Silenciar",
        (Idioma::Ruso, "mute") => "Без звука",
        (Idioma::Sueco, "mute") => "Ljud av",
        (Idioma::Turco, "mute") => "Sessiz",
        (Idioma::Ucraniano, "mute") => "Вимкнути звук",
        (Idioma::Vietnamita, "mute") => "Tắt tiếng",
        (Idioma::ChinoSimplificado, "mute") => "静音",
        (Idioma::Ingles, "cameras") => "Cameras",
        (Idioma::BelarusLatino, "cameras") => "Kamjery",
        (Idioma::Belarus, "cameras") => "Камеры",
        (Idioma::Catalan, "cameras") => "Càmeres",
        (Idioma::Checo, "cameras") => "Fotoaparáty",
        (Idioma::Aleman, "cameras") => "Kameras",
        (Idioma::Frances, "cameras") => "Caméras",
        (Idioma::Gallego, "cameras") => "Cámaras",
        (Idioma::Italiano, "cameras") => "Telecamere",
        (Idioma::Coreano, "cameras") => "카메라",
        (Idioma::Kurdo, "cameras") => "Kamera",
        (Idioma::Neerlandes, "cameras") => "Camera's",
        (Idioma::NoruegoNynorsk, "cameras") => "Kameraer",
        (Idioma::Polaco, "cameras") => "Kamery",
        (Idioma::PortuguesBrasil, "cameras") => "Câmeras",
        (Idioma::Ruso, "cameras") => "Камеры",
        (Idioma::Sueco, "cameras") => "Kameror",
        (Idioma::Turco, "cameras") => "Kameralar",
        (Idioma::Ucraniano, "cameras") => "Камери",
        (Idioma::Vietnamita, "cameras") => "Máy Ảnh",
        (Idioma::ChinoSimplificado, "cameras") => "相机",
        (Idioma::Ingles, "software_sources") => "Software catalog",
        (Idioma::BelarusLatino, "software_sources") => "Katałoh prahramnaha zabjespjačennja",
        (Idioma::Belarus, "software_sources") => "Каталог праграмнага забеспячэння",
        (Idioma::Catalan, "software_sources") => "Catàleg de programari",
        (Idioma::Checo, "software_sources") => "Katalog softwaru",
        (Idioma::Aleman, "software_sources") => "Softwarekatalog",
        (Idioma::Frances, "software_sources") => "Catalogue de logiciels",
        (Idioma::Gallego, "software_sources") => "Catálogo de software",
        (Idioma::Italiano, "software_sources") => "Catalogo software",
        (Idioma::Coreano, "software_sources") => "소프트웨어 카탈로그",
        (Idioma::Kurdo, "software_sources") => "Kataloga nermalavê",
        (Idioma::Neerlandes, "software_sources") => "Softwarecatalogus",
        (Idioma::NoruegoNynorsk, "software_sources") => "Programvarekatalog",
        (Idioma::Polaco, "software_sources") => "Katalog oprogramowania",
        (Idioma::PortuguesBrasil, "software_sources") => "Catálogo de software",
        (Idioma::Ruso, "software_sources") => "Каталог программного обеспечения",
        (Idioma::Sueco, "software_sources") => "Programvarukatalog",
        (Idioma::Turco, "software_sources") => "Yazılım kataloğu",
        (Idioma::Ucraniano, "software_sources") => "Каталог програмного забезпечення",
        (Idioma::Vietnamita, "software_sources") => "Danh mục phần mềm",
        (Idioma::ChinoSimplificado, "software_sources") => "软件目录",
        (Idioma::Ingles, "update_sources") => "Update catalog",
        (Idioma::BelarusLatino, "update_sources") => "Abnaŭłjennje katałoha",
        (Idioma::Belarus, "update_sources") => "Абнаўленне каталога",
        (Idioma::Catalan, "update_sources") => "Actualitza el catàleg",
        (Idioma::Checo, "update_sources") => "Aktualizace katalogu",
        (Idioma::Aleman, "update_sources") => "Katalog aktualisieren",
        (Idioma::Frances, "update_sources") => "Mettre à jour le catalogue",
        (Idioma::Gallego, "update_sources") => "Actualizar catálogo",
        (Idioma::Italiano, "update_sources") => "Aggiorna catalogo",
        (Idioma::Coreano, "update_sources") => "카탈로그 업데이트",
        (Idioma::Kurdo, "update_sources") => "Nûvekirina katalogê",
        (Idioma::Neerlandes, "update_sources") => "Catalogus bijwerken",
        (Idioma::NoruegoNynorsk, "update_sources") => "Oppdater katalog",
        (Idioma::Polaco, "update_sources") => "Aktualizuj katalog",
        (Idioma::PortuguesBrasil, "update_sources") => "Atualizar catálogo",
        (Idioma::Ruso, "update_sources") => "Обновление каталога",
        (Idioma::Sueco, "update_sources") => "Uppdatera katalog",
        (Idioma::Turco, "update_sources") => "Kataloğu güncelle",
        (Idioma::Ucraniano, "update_sources") => "Оновити каталог",
        (Idioma::Vietnamita, "update_sources") => "Cập nhật danh mục",
        (Idioma::ChinoSimplificado, "update_sources") => "更新目录",
        (Idioma::Ingles, "privileges") => "Authorization",
        (Idioma::BelarusLatino, "privileges") => "Aŭtaryzacyja",
        (Idioma::Belarus, "privileges") => "Аўтарызацыя",
        (Idioma::Catalan, "privileges") => "Autorització",
        (Idioma::Checo, "privileges") => "Autorizace",
        (Idioma::Aleman, "privileges") => "Autorisierung",
        (Idioma::Frances, "privileges") => "Autorisation",
        (Idioma::Gallego, "privileges") => "Autorización",
        (Idioma::Italiano, "privileges") => "Autorizzazione",
        (Idioma::Coreano, "privileges") => "승인",
        (Idioma::Kurdo, "privileges") => "Destûrkirin",
        (Idioma::Neerlandes, "privileges") => "Autorisatie",
        (Idioma::NoruegoNynorsk, "privileges") => "Autorisasjon",
        (Idioma::Polaco, "privileges") => "Autoryzacja",
        (Idioma::PortuguesBrasil, "privileges") => "Autorização",
        (Idioma::Ruso, "privileges") => "Авторизация",
        (Idioma::Sueco, "privileges") => "Auktorisering",
        (Idioma::Turco, "privileges") => "Authorization",
        (Idioma::Ucraniano, "privileges") => "Авторизація",
        (Idioma::Vietnamita, "privileges") => "Ủy quyền",
        (Idioma::ChinoSimplificado, "privileges") => "授权",
        (Idioma::Ingles, "operation_done") => "Operation completed.",
        (Idioma::BelarusLatino, "operation_done") => "Apjeracyja zavjeršana.",
        (Idioma::Belarus, "operation_done") => "Аперацыя завершана.",
        (Idioma::Catalan, "operation_done") => "Operació completada.",
        (Idioma::Checo, "operation_done") => "Operace dokončena.",
        (Idioma::Aleman, "operation_done") => "Vorgang abgeschlossen.",
        (Idioma::Frances, "operation_done") => "Opération terminée.",
        (Idioma::Gallego, "operation_done") => "Operación rematada.",
        (Idioma::Italiano, "operation_done") => "Operazione completata.",
        (Idioma::Coreano, "operation_done") => "작업이 완료되었습니다.",
        (Idioma::Kurdo, "operation_done") => "Operasyon qediya.",
        (Idioma::Neerlandes, "operation_done") => "Bewerking voltooid.",
        (Idioma::NoruegoNynorsk, "operation_done") => "Operasjon fullført.",
        (Idioma::Polaco, "operation_done") => "Operacja zakończona.",
        (Idioma::PortuguesBrasil, "operation_done") => "Operação concluída.",
        (Idioma::Ruso, "operation_done") => "Операция завершена.",
        (Idioma::Sueco, "operation_done") => "Operation slutförd.",
        (Idioma::Turco, "operation_done") => "İşlem tamamlandı.",
        (Idioma::Ucraniano, "operation_done") => "Операцію завершено.",
        (Idioma::Vietnamita, "operation_done") => "Hoạt động đã hoàn tất.",
        (Idioma::ChinoSimplificado, "operation_done") => "操作完成。",
        (Idioma::Ingles, "operation_failed") => "The operation failed.",
        (Idioma::BelarusLatino, "operation_failed") => "Apjeracyja pravałiłasja.",
        (Idioma::Belarus, "operation_failed") => "Аперацыя не атрымалася.",
        (Idioma::Catalan, "operation_failed") => "L'operació ha fallat.",
        (Idioma::Checo, "operation_failed") => "Operace se nezdařila.",
        (Idioma::Aleman, "operation_failed") => "Der Vorgang ist fehlgeschlagen.",
        (Idioma::Frances, "operation_failed") => "L'opération a échoué.",
        (Idioma::Gallego, "operation_failed") => "Fallou a operación.",
        (Idioma::Italiano, "operation_failed") => "Operazione non riuscita.",
        (Idioma::Coreano, "operation_failed") => "작업이 실패했습니다.",
        (Idioma::Kurdo, "operation_failed") => "Operasyon bi ser neket.",
        (Idioma::Neerlandes, "operation_failed") => "De bewerking is mislukt.",
        (Idioma::NoruegoNynorsk, "operation_failed") => "Operasjonen mislyktes.",
        (Idioma::Polaco, "operation_failed") => "Operacja nie powiodła się.",
        (Idioma::PortuguesBrasil, "operation_failed") => "A operação falhou.",
        (Idioma::Ruso, "operation_failed") => "Операция не удалась.",
        (Idioma::Sueco, "operation_failed") => "Åtgärden misslyckades.",
        (Idioma::Turco, "operation_failed") => "İşlem başarısız oldu.",
        (Idioma::Ucraniano, "operation_failed") => "Не вдалося виконати операцію.",
        (Idioma::Vietnamita, "operation_failed") => "Thao tác không thành công.",
        (Idioma::ChinoSimplificado, "operation_failed") => "操作失败。",
        (Idioma::Ingles, "confirm_operation") => "Confirm operation",
        (Idioma::BelarusLatino, "confirm_operation") => "Pacvjerdzic apjeracyju",
        (Idioma::Belarus, "confirm_operation") => "Пацвердзіць аперацыю",
        (Idioma::Catalan, "confirm_operation") => "Confirmeu l'operació",
        (Idioma::Checo, "confirm_operation") => "Potvrďte operaci",
        (Idioma::Aleman, "confirm_operation") => "Bestätigen Sie den Vorgang",
        (Idioma::Frances, "confirm_operation") => "Confirmer l'opération",
        (Idioma::Gallego, "confirm_operation") => "Confirmar a operación",
        (Idioma::Italiano, "confirm_operation") => "Conferma l'operazione",
        (Idioma::Coreano, "confirm_operation") => "작업 확인",
        (Idioma::Kurdo, "confirm_operation") => "Operasyonê piştrast bike",
        (Idioma::Neerlandes, "confirm_operation") => "Bewerking bevestigen",
        (Idioma::NoruegoNynorsk, "confirm_operation") => "Bekreft operasjon",
        (Idioma::Polaco, "confirm_operation") => "Potwierdź operację",
        (Idioma::PortuguesBrasil, "confirm_operation") => "Confirmar operação",
        (Idioma::Ruso, "confirm_operation") => "Подтвердить операцию",
        (Idioma::Sueco, "confirm_operation") => "Bekräfta åtgärden",
        (Idioma::Turco, "confirm_operation") => "İşlemi onayla",
        (Idioma::Ucraniano, "confirm_operation") => "Підтвердити операцію",
        (Idioma::Vietnamita, "confirm_operation") => "Xác nhận thao tác",
        (Idioma::ChinoSimplificado, "confirm_operation") => "确认操作",
        (Idioma::Ingles, "current") => "Current",
        (Idioma::BelarusLatino, "current") => "Tok",
        (Idioma::Belarus, "current") => "Ток",
        (Idioma::Catalan, "current") => "Actual",
        (Idioma::Checo, "current") => "Aktuální",
        (Idioma::Aleman, "current") => "Aktuell",
        (Idioma::Frances, "current") => "Actuel",
        (Idioma::Gallego, "current") => "Actual",
        (Idioma::Italiano, "current") => "Corrente",
        (Idioma::Coreano, "current") => "현재",
        (Idioma::Kurdo, "current") => "Niha",
        (Idioma::Neerlandes, "current") => "Huidig",
        (Idioma::NoruegoNynorsk, "current") => "Gjeldende",
        (Idioma::Polaco, "current") => "Prąd",
        (Idioma::PortuguesBrasil, "current") => "Atual",
        (Idioma::Ruso, "current") => "Текущий",
        (Idioma::Sueco, "current") => "Aktuell",
        (Idioma::Turco, "current") => "Güncel",
        (Idioma::Ucraniano, "current") => "Поточний",
        (Idioma::Vietnamita, "current") => "hiện tại",
        (Idioma::ChinoSimplificado, "current") => "当前",
        (Idioma::Ingles, "host") => "Computer",
        (Idioma::BelarusLatino, "host") => "Kamputar",
        (Idioma::Belarus, "host") => "Кампутар",
        (Idioma::Catalan, "host") => "Ordinador",
        (Idioma::Checo, "host") => "Počítač",
        (Idioma::Aleman, "host") => "Computer",
        (Idioma::Frances, "host") => "Ordinateur",
        (Idioma::Gallego, "host") => "Ordenador",
        (Idioma::Italiano, "host") => "Computer",
        (Idioma::Coreano, "host") => "컴퓨터",
        (Idioma::Kurdo, "host") => "Komputer",
        (Idioma::Neerlandes, "host") => "Computer",
        (Idioma::NoruegoNynorsk, "host") => "Datamaskin",
        (Idioma::Polaco, "host") => "Komputer",
        (Idioma::PortuguesBrasil, "host") => "Computador",
        (Idioma::Ruso, "host") => "Компьютер",
        (Idioma::Sueco, "host") => "Dator",
        (Idioma::Turco, "host") => "Bilgisayar",
        (Idioma::Ucraniano, "host") => "Комп'ютер",
        (Idioma::Vietnamita, "host") => "Máy tính",
        (Idioma::ChinoSimplificado, "host") => "电脑",
        (Idioma::Ingles, "model") => "Model",
        (Idioma::BelarusLatino, "model") => "madeł",
        (Idioma::Belarus, "model") => "Мадэль",
        (Idioma::Catalan, "model") => "Model",
        (Idioma::Checo, "model") => "Model",
        (Idioma::Aleman, "model") => "Modell",
        (Idioma::Frances, "model") => "Modèle",
        (Idioma::Gallego, "model") => "Modelo",
        (Idioma::Italiano, "model") => "Modello",
        (Idioma::Coreano, "model") => "모델",
        (Idioma::Kurdo, "model") => "Model",
        (Idioma::Neerlandes, "model") => "Model",
        (Idioma::NoruegoNynorsk, "model") => "Modell",
        (Idioma::Polaco, "model") => "Model",
        (Idioma::PortuguesBrasil, "model") => "Modelo",
        (Idioma::Ruso, "model") => "Модель",
        (Idioma::Sueco, "model") => "Modell",
        (Idioma::Turco, "model") => "Modeli",
        (Idioma::Ucraniano, "model") => "Модель",
        (Idioma::Vietnamita, "model") => "Model",
        (Idioma::ChinoSimplificado, "model") => "型号",
        (Idioma::Ingles, "cpu") => "Processor",
        (Idioma::BelarusLatino, "cpu") => "Pracesar",
        (Idioma::Belarus, "cpu") => "Працэсар",
        (Idioma::Catalan, "cpu") => "Processador",
        (Idioma::Checo, "cpu") => "Procesor",
        (Idioma::Aleman, "cpu") => "Prozessor",
        (Idioma::Frances, "cpu") => "Processeur",
        (Idioma::Gallego, "cpu") => "Procesador",
        (Idioma::Italiano, "cpu") => "Processore",
        (Idioma::Coreano, "cpu") => "프로세서",
        (Idioma::Kurdo, "cpu") => "Pêvajoya",
        (Idioma::Neerlandes, "cpu") => "Processor",
        (Idioma::NoruegoNynorsk, "cpu") => "Prosessor",
        (Idioma::Polaco, "cpu") => "Procesor",
        (Idioma::PortuguesBrasil, "cpu") => "Processador",
        (Idioma::Ruso, "cpu") => "Процессор",
        (Idioma::Sueco, "cpu") => "Processor",
        (Idioma::Turco, "cpu") => "İşlemci",
        (Idioma::Ucraniano, "cpu") => "Процесор",
        (Idioma::Vietnamita, "cpu") => "Bộ xử lý",
        (Idioma::ChinoSimplificado, "cpu") => "处理器",
        (Idioma::Ingles, "memory") => "Memory",
        (Idioma::BelarusLatino, "memory") => "Pamjac",
        (Idioma::Belarus, "memory") => "Памяць",
        (Idioma::Catalan, "memory") => "Memòria",
        (Idioma::Checo, "memory") => "Paměť",
        (Idioma::Aleman, "memory") => "Speicher",
        (Idioma::Frances, "memory") => "Mémoire",
        (Idioma::Gallego, "memory") => "Memoria",
        (Idioma::Italiano, "memory") => "Memoria",
        (Idioma::Coreano, "memory") => "메모리",
        (Idioma::Kurdo, "memory") => "Bîrdank",
        (Idioma::Neerlandes, "memory") => "Geheugen",
        (Idioma::NoruegoNynorsk, "memory") => "Minne",
        (Idioma::Polaco, "memory") => "Pamięć",
        (Idioma::PortuguesBrasil, "memory") => "Memória",
        (Idioma::Ruso, "memory") => "Память",
        (Idioma::Sueco, "memory") => "Minne",
        (Idioma::Turco, "memory") => "Bellek",
        (Idioma::Ucraniano, "memory") => "Пам'ять",
        (Idioma::Vietnamita, "memory") => "RAM",
        (Idioma::ChinoSimplificado, "memory") => "内存",
        (Idioma::Ingles, "boot") => "Boot type",
        (Idioma::BelarusLatino, "boot") => "Typ zahruzki",
        (Idioma::Belarus, "boot") => "Тып загрузкі",
        (Idioma::Catalan, "boot") => "Tipus d'arrencada",
        (Idioma::Checo, "boot") => "Typ spouštění",
        (Idioma::Aleman, "boot") => "Boot-Typ",
        (Idioma::Frances, "boot") => "Type de démarrage",
        (Idioma::Gallego, "boot") => "Tipo de inicio",
        (Idioma::Italiano, "boot") => "Tipo di avvio",
        (Idioma::Coreano, "boot") => "부팅 유형",
        (Idioma::Kurdo, "boot") => "Cureyê bootê",
        (Idioma::Neerlandes, "boot") => "Opstarttype",
        (Idioma::NoruegoNynorsk, "boot") => "Oppstartstype",
        (Idioma::Polaco, "boot") => "Typ rozruchu",
        (Idioma::PortuguesBrasil, "boot") => "Tipo de inicialização",
        (Idioma::Ruso, "boot") => "Тип загрузки",
        (Idioma::Sueco, "boot") => "Starttyp",
        (Idioma::Turco, "boot") => "Önyükleme türü",
        (Idioma::Ucraniano, "boot") => "Тип завантаження",
        (Idioma::Vietnamita, "boot") => "Loại khởi động",
        (Idioma::ChinoSimplificado, "boot") => "引导类型",
        (Idioma::Ingles, "applied") => "Change applied.",
        (Idioma::BelarusLatino, "applied") => "Zmjena prymjenjena.",
        (Idioma::Belarus, "applied") => "Змена прыменена.",
        (Idioma::Catalan, "applied") => "S'ha aplicat el canvi.",
        (Idioma::Checo, "applied") => "Změna byla použita.",
        (Idioma::Aleman, "applied") => "Änderung angewendet.",
        (Idioma::Frances, "applied") => "Modification appliquée.",
        (Idioma::Gallego, "applied") => "Cambio aplicado.",
        (Idioma::Italiano, "applied") => "Modifica applicata.",
        (Idioma::Coreano, "applied") => "변경 사항이 적용되었습니다.",
        (Idioma::Kurdo, "applied") => "Guhertin hate sepandin.",
        (Idioma::Neerlandes, "applied") => "Wijziging toegepast.",
        (Idioma::NoruegoNynorsk, "applied") => "Endring tatt i bruk.",
        (Idioma::Polaco, "applied") => "Zmiana została zastosowana.",
        (Idioma::PortuguesBrasil, "applied") => "Alteração aplicada.",
        (Idioma::Ruso, "applied") => "Изменение применено.",
        (Idioma::Sueco, "applied") => "Ändring tillämpad.",
        (Idioma::Turco, "applied") => "Change applied.",
        (Idioma::Ucraniano, "applied") => "Зміни застосовано.",
        (Idioma::Vietnamita, "applied") => "Đã áp dụng thay đổi.",
        (Idioma::ChinoSimplificado, "applied") => "已应用更改。",
        (Idioma::Ingles, "change_failed") => "The change could not be applied.",
        (Idioma::BelarusLatino, "change_failed") => "Nje ŭdałosja ŭžyc zmjanjennje.",
        (Idioma::Belarus, "change_failed") => "Немагчыма прымяніць змяненне.",
        (Idioma::Catalan, "change_failed") => "El canvi no s'ha pogut aplicar.",
        (Idioma::Checo, "change_failed") => "Změnu nelze použít.",
        (Idioma::Aleman, "change_failed") => "Die Änderung konnte nicht angewendet werden.",
        (Idioma::Frances, "change_failed") => "La modification n'a pas pu être appliquée.",
        (Idioma::Gallego, "change_failed") => "Non se puido aplicar o cambio.",
        (Idioma::Italiano, "change_failed") => "Impossibile applicare la modifica.",
        (Idioma::Coreano, "change_failed") => "변경 사항을 적용할 수 없습니다.",
        (Idioma::Kurdo, "change_failed") => "Guhertin nehat sepandin.",
        (Idioma::Neerlandes, "change_failed") => "De wijziging kon niet worden toegepast.",
        (Idioma::NoruegoNynorsk, "change_failed") => "Endringen kunne ikke brukes.",
        (Idioma::Polaco, "change_failed") => "Nie można zastosować zmiany.",
        (Idioma::PortuguesBrasil, "change_failed") => "A alteração não pôde ser aplicada.",
        (Idioma::Ruso, "change_failed") => "Не удалось применить изменение.",
        (Idioma::Sueco, "change_failed") => "Ändringen kunde inte tillämpas.",
        (Idioma::Turco, "change_failed") => "Değişiklik uygulanamadı.",
        (Idioma::Ucraniano, "change_failed") => "Не вдалося застосувати зміну.",
        (Idioma::Vietnamita, "change_failed") => "Không thể áp dụng thay đổi.",
        (Idioma::ChinoSimplificado, "change_failed") => "无法应用更改。",
        (Idioma::Ingles, "language") => "Language",
        (Idioma::BelarusLatino, "language") => "mova",
        (Idioma::Belarus, "language") => "Мова",
        (Idioma::Catalan, "language") => "Idioma",
        (Idioma::Checo, "language") => "Jazyk",
        (Idioma::Aleman, "language") => "Sprache",
        (Idioma::Frances, "language") => "Langue",
        (Idioma::Gallego, "language") => "Idioma",
        (Idioma::Italiano, "language") => "Lingua",
        (Idioma::Coreano, "language") => "언어",
        (Idioma::Kurdo, "language") => "Ziman",
        (Idioma::Neerlandes, "language") => "Taal",
        (Idioma::NoruegoNynorsk, "language") => "Mål",
        (Idioma::Polaco, "language") => "Język",
        (Idioma::PortuguesBrasil, "language") => "Idioma",
        (Idioma::Ruso, "language") => "Язык",
        (Idioma::Sueco, "language") => "Språk",
        (Idioma::Turco, "language") => "Dil",
        (Idioma::Ucraniano, "language") => "Мова",
        (Idioma::Vietnamita, "language") => "Ngôn ngữ",
        (Idioma::ChinoSimplificado, "language") => "语言",
        (Idioma::Ingles, "region") => "Region",
        (Idioma::BelarusLatino, "region") => "Rehijon",
        (Idioma::Belarus, "region") => "Рэг",
        (Idioma::Catalan, "region") => "Regió",
        (Idioma::Checo, "region") => "Region",
        (Idioma::Aleman, "region") => "Region",
        (Idioma::Frances, "region") => "Région",
        (Idioma::Gallego, "region") => "Rexión",
        (Idioma::Italiano, "region") => "Regione",
        (Idioma::Coreano, "region") => "지역",
        (Idioma::Kurdo, "region") => "Herêm",
        (Idioma::Neerlandes, "region") => "Regio",
        (Idioma::NoruegoNynorsk, "region") => "Region",
        (Idioma::Polaco, "region") => "Region",
        (Idioma::PortuguesBrasil, "region") => "Região",
        (Idioma::Ruso, "region") => "Регион",
        (Idioma::Sueco, "region") => "Region",
        (Idioma::Turco, "region") => "Bölge",
        (Idioma::Ucraniano, "region") => "Регіон",
        (Idioma::Vietnamita, "region") => "Khu vực",
        (Idioma::ChinoSimplificado, "region") => "地区",
        (Idioma::Ingles, "timezone") => "Time zone",
        (Idioma::BelarusLatino, "timezone") => "Časavy pojas",
        (Idioma::Belarus, "timezone") => "Часавы пояс",
        (Idioma::Catalan, "timezone") => "Zona horària",
        (Idioma::Checo, "timezone") => "Časové pásmo",
        (Idioma::Aleman, "timezone") => "Zeitzone",
        (Idioma::Frances, "timezone") => "Fuseau horaire",
        (Idioma::Gallego, "timezone") => "Fuso horario",
        (Idioma::Italiano, "timezone") => "Fuso orario",
        (Idioma::Coreano, "timezone") => "시간대",
        (Idioma::Kurdo, "timezone") => "Demjimêr",
        (Idioma::Neerlandes, "timezone") => "Tijdzone",
        (Idioma::NoruegoNynorsk, "timezone") => "Tidssone",
        (Idioma::Polaco, "timezone") => "Strefa czasowa",
        (Idioma::PortuguesBrasil, "timezone") => "Fuso horário",
        (Idioma::Ruso, "timezone") => "Часовой пояс",
        (Idioma::Sueco, "timezone") => "Tidszon",
        (Idioma::Turco, "timezone") => "Saat dilimi",
        (Idioma::Ucraniano, "timezone") => "Часовий пояс",
        (Idioma::Vietnamita, "timezone") => "Múi giờ",
        (Idioma::ChinoSimplificado, "timezone") => "时区",
        (Idioma::Ingles, "keyboard") => "Keyboard",
        (Idioma::BelarusLatino, "keyboard") => "Kłavijatura",
        (Idioma::Belarus, "keyboard") => "Клавіятура",
        (Idioma::Catalan, "keyboard") => "Teclat",
        (Idioma::Checo, "keyboard") => "Klávesnice",
        (Idioma::Aleman, "keyboard") => "Tastatur",
        (Idioma::Frances, "keyboard") => "Clavier",
        (Idioma::Gallego, "keyboard") => "Teclado",
        (Idioma::Italiano, "keyboard") => "Tastiera",
        (Idioma::Coreano, "keyboard") => "키보드",
        (Idioma::Kurdo, "keyboard") => "Klavyeya",
        (Idioma::Neerlandes, "keyboard") => "Toetsenbord",
        (Idioma::NoruegoNynorsk, "keyboard") => "Tastatur",
        (Idioma::Polaco, "keyboard") => "Klawiatura",
        (Idioma::PortuguesBrasil, "keyboard") => "Teclado",
        (Idioma::Ruso, "keyboard") => "Клавиатура",
        (Idioma::Sueco, "keyboard") => "Tangentbord",
        (Idioma::Turco, "keyboard") => "Klavye",
        (Idioma::Ucraniano, "keyboard") => "Клавіатура",
        (Idioma::Vietnamita, "keyboard") => "Bàn phím",
        (Idioma::ChinoSimplificado, "keyboard") => "键盘",
        (Idioma::Ingles, "status") => "Status",
        (Idioma::BelarusLatino, "status") => "Status",
        (Idioma::Belarus, "status") => "Стан",
        (Idioma::Catalan, "status") => "Estat",
        (Idioma::Checo, "status") => "Stav",
        (Idioma::Aleman, "status") => "Status",
        (Idioma::Frances, "status") => "Status",
        (Idioma::Gallego, "status") => "Estado",
        (Idioma::Italiano, "status") => "Stato",
        (Idioma::Coreano, "status") => "상태",
        (Idioma::Kurdo, "status") => "Rewş",
        (Idioma::Neerlandes, "status") => "Status",
        (Idioma::NoruegoNynorsk, "status") => "Status",
        (Idioma::Polaco, "status") => "Status",
        (Idioma::PortuguesBrasil, "status") => "Status",
        (Idioma::Ruso, "status") => "Статус",
        (Idioma::Sueco, "status") => "Status",
        (Idioma::Turco, "status") => "Durum",
        (Idioma::Ucraniano, "status") => "Стан",
        (Idioma::Vietnamita, "status") => "Trạng thái",
        (Idioma::ChinoSimplificado, "status") => "状态",
        (Idioma::Ingles, "loading") => "Reading this computer…",
        (Idioma::BelarusLatino, "loading") => "Čytannje hetaha kamp'jutara...",
        (Idioma::Belarus, "loading") => "Чытанне гэтага кампутара...",
        (Idioma::Catalan, "loading") => "Llegint aquest ordinador...",
        (Idioma::Checo, "loading") => "Čtení tohoto počítače…",
        (Idioma::Aleman, "loading") => "Dieser Computer wird gelesen…",
        (Idioma::Frances, "loading") => "Lecture de cet ordinateur…",
        (Idioma::Gallego, "loading") => "Lendo este ordenador…",
        (Idioma::Italiano, "loading") => "Lettura di questo computer…",
        (Idioma::Coreano, "loading") => "이 컴퓨터를 읽는 중…",
        (Idioma::Kurdo, "loading") => "Xwendina vê kompîturê…",
        (Idioma::Neerlandes, "loading") => "Deze computer lezen…",
        (Idioma::NoruegoNynorsk, "loading") => "Leser denne datamaskinen...",
        (Idioma::Polaco, "loading") => "Czytanie tego komputera…",
        (Idioma::PortuguesBrasil, "loading") => "Lendo este computador…",
        (Idioma::Ruso, "loading") => "Читаю этот компьютер…",
        (Idioma::Sueco, "loading") => "Läser den här datorn...",
        (Idioma::Turco, "loading") => "Bu bilgisayar okunuyor…",
        (Idioma::Ucraniano, "loading") => "Читання цього комп’ютера…",
        (Idioma::Vietnamita, "loading") => "Đang đọc máy tính này…",
        (Idioma::ChinoSimplificado, "loading") => "正在读取这台计算机...",
        (Idioma::Ingles, "ready") => "Ready",
        (Idioma::BelarusLatino, "ready") => "Hatovy",
        (Idioma::Belarus, "ready") => "Гатова",
        (Idioma::Catalan, "ready") => "A punt",
        (Idioma::Checo, "ready") => "Připraveno",
        (Idioma::Aleman, "ready") => "Bereit",
        (Idioma::Frances, "ready") => "Prêt",
        (Idioma::Gallego, "ready") => "Listo",
        (Idioma::Italiano, "ready") => "Pronto",
        (Idioma::Coreano, "ready") => "준비됨",
        (Idioma::Kurdo, "ready") => "Amade ye",
        (Idioma::Neerlandes, "ready") => "Klaar",
        (Idioma::NoruegoNynorsk, "ready") => "Klar",
        (Idioma::Polaco, "ready") => "Gotowe",
        (Idioma::PortuguesBrasil, "ready") => "Pronto",
        (Idioma::Ruso, "ready") => "Готово",
        (Idioma::Sueco, "ready") => "Klar",
        (Idioma::Turco, "ready") => "Hazır",
        (Idioma::Ucraniano, "ready") => "Готовий",
        (Idioma::Vietnamita, "ready") => "Sẵn sàng",
        (Idioma::ChinoSimplificado, "ready") => "准备好",
        (Idioma::Ingles, "error") => "Korunix could not read this area.",
        (Idioma::BelarusLatino, "error") => "Korunix nje moža pračytac hetuju vobłasc.",
        (Idioma::Belarus, "error") => "Korunix не можа прачытаць гэтую вобласць.",
        (Idioma::Catalan, "error") => "Korunix no ha pogut llegir aquesta àrea.",
        (Idioma::Checo, "error") => "Korunix nemohl přečíst tuto oblast.",
        (Idioma::Aleman, "error") => "Korunix konnte diesen Bereich nicht lesen.",
        (Idioma::Frances, "error") => "Korunix n'a pas pu lire cette zone.",
        (Idioma::Gallego, "error") => "Korunix non puido ler esta área.",
        (Idioma::Italiano, "error") => "Korunix non è riuscito a leggere quest'area.",
        (Idioma::Coreano, "error") => "Korunix가 이 영역을 읽을 수 없습니다.",
        (Idioma::Kurdo, "error") => "Korunixê nikarîbû vê deverê bixwîne.",
        (Idioma::Neerlandes, "error") => "Korunix kon dit gebied niet lezen.",
        (Idioma::NoruegoNynorsk, "error") => "Korunix kunne ikke lese dette området.",
        (Idioma::Polaco, "error") => "Korunix nie mógł odczytać tego obszaru.",
        (Idioma::PortuguesBrasil, "error") => "Korunix não conseguiu ler esta área.",
        (Idioma::Ruso, "error") => "Korunix не смог прочитать эту область.",
        (Idioma::Sueco, "error") => "Korunix kunde inte läsa detta område.",
        (Idioma::Turco, "error") => "Korunix bu alanı okuyamadı.",
        (Idioma::Ucraniano, "error") => "Korunix не може прочитати цю область.",
        (Idioma::Vietnamita, "error") => "Korunix không thể đọc được khu vực này.",
        (Idioma::ChinoSimplificado, "error") => "Korunix 无法读取该区域。",
        (Idioma::Ingles, "empty") => "No information available.",
        (Idioma::BelarusLatino, "empty") => "Njama infarmacyi.",
        (Idioma::Belarus, "empty") => "Інфармацыя адсутнічае.",
        (Idioma::Catalan, "empty") => "No hi ha informació disponible.",
        (Idioma::Checo, "empty") => "Žádná informace není k dispozici.",
        (Idioma::Aleman, "empty") => "Keine Informationen verfügbar.",
        (Idioma::Frances, "empty") => "Aucune information disponible.",
        (Idioma::Gallego, "empty") => "Non hai información dispoñible.",
        (Idioma::Italiano, "empty") => "Nessuna informazione disponibile.",
        (Idioma::Coreano, "empty") => "사용 가능한 정보가 없습니다.",
        (Idioma::Kurdo, "empty") => "Agahî tune.",
        (Idioma::Neerlandes, "empty") => "Geen informatie beschikbaar.",
        (Idioma::NoruegoNynorsk, "empty") => "Ingen informasjon tilgjengelig.",
        (Idioma::Polaco, "empty") => "Brak dostępnych informacji.",
        (Idioma::PortuguesBrasil, "empty") => "Nenhuma informação disponível.",
        (Idioma::Ruso, "empty") => "Информация отсутствует.",
        (Idioma::Sueco, "empty") => "Ingen information tillgänglig.",
        (Idioma::Turco, "empty") => "Bilgi mevcut değil.",
        (Idioma::Ucraniano, "empty") => "Інформація відсутня.",
        (Idioma::Vietnamita, "empty") => "Không có thông tin.",
        (Idioma::ChinoSimplificado, "empty") => "无可用信息。",

        (Idioma::Hungaro, "subtitle") => "NixOS vezérlőközpont",
        (Idioma::Hungaro, "summary") => "Összefoglaló",
        (Idioma::Hungaro, "updates") => "Frissítések",
        (Idioma::Hungaro, "localization") => "Nyelv és régió",
        (Idioma::Hungaro, "hardware") => "Hardver",
        (Idioma::Hungaro, "people") => "Személyek",
        (Idioma::Hungaro, "refresh") => "Frissítés",
        (Idioma::Hungaro, "channel") => "Rendszercsatorna",
        (Idioma::Hungaro, "stable") => "Stabil",
        (Idioma::Hungaro, "unstable") => "Instabil",
        (Idioma::Hungaro, "prepare") => "Változtatás előkészítése",
        (Idioma::Hungaro, "configured") => "Beállítva",
        (Idioma::Hungaro, "target_channel") => "Új csatorna",
        (Idioma::Hungaro, "change") => "Módosítás",
        (Idioma::Hungaro, "confirm_channel") => "Csatornaváltás megerősítése",
        (Idioma::Hungaro, "cancel") => "Mégse",
        (Idioma::Hungaro, "apply_change") => "Módosítás alkalmazása",
        (Idioma::Hungaro, "no_change") => "Előbb válassz másik csatornát.",
        (Idioma::Hungaro, "plan_failed") => "A módosítás előnézetét nem sikerült elkészíteni.",
        (Idioma::Hungaro, "sections") => "Szakaszok",
        (Idioma::Hungaro, "media") => "Hang és kamera",
        (Idioma::Hungaro, "storage") => "Tárhely",
        (Idioma::Hungaro, "firmware_updates") => "Firmware",
        (Idioma::Hungaro, "maintenance") => "Karbantartás",
        (Idioma::Hungaro, "recovery") => "Helyreállítási verziók",
        (Idioma::Hungaro, "cleanup") => "Tisztítás",
        (Idioma::Hungaro, "normal_cleanup") => "Ajánlott tisztítás",
        (Idioma::Hungaro, "deep_cleanup") => "Alapos tisztítás",
        (Idioma::Hungaro, "clean_now") => "Tisztítás",
        (Idioma::Hungaro, "clean_all") => "Több törlése",
        (Idioma::Hungaro, "generations") => "Elérhető verziók",
        (Idioma::Hungaro, "use_once") => "Próba a következő újraindításkor",
        (Idioma::Hungaro, "current_generation") => "Használt verzió",
        (Idioma::Hungaro, "default_generation") => "Indításkor használt verzió",
        (Idioma::Hungaro, "heavy_transfer") => "Várakozás az adatok mentésének befejezésére",
        (Idioma::Hungaro, "heavy_transfer_detail") => {
            "Leválasztás előtt várd meg, amíg minden adat mentése befejeződik a meghajtón."
        }
        (Idioma::Hungaro, "eject") => "Kiadás",
        (Idioma::Hungaro, "removable") => "Eltávolítható",
        (Idioma::Hungaro, "internal") => "Belső",
        (Idioma::Hungaro, "safe_disconnect") => "Biztonságosan leválasztható.",
        (Idioma::Hungaro, "available_updates") => "Firmware-frissítések",
        (Idioma::Hungaro, "firmware_devices") => "Firmware-eszközök",
        (Idioma::Hungaro, "refresh_firmware") => "Frissítések ellenőrzése",
        (Idioma::Hungaro, "install") => "Telepítés",
        (Idioma::Hungaro, "no_updates") => "Nincs elérhető frissítés.",
        (Idioma::Hungaro, "version") => "Verzió",
        (Idioma::Hungaro, "output") => "Hangkimenet",
        (Idioma::Hungaro, "input") => "Mikrofon",
        (Idioma::Hungaro, "volume") => "Hangerő",
        (Idioma::Hungaro, "mute") => "Némítás",
        (Idioma::Hungaro, "cameras") => "Kamerák",
        (Idioma::Hungaro, "software_sources") => "Szoftverkatalógus",
        (Idioma::Hungaro, "update_sources") => "Katalógus frissítése",
        (Idioma::Hungaro, "privileges") => "Engedélyezés",
        (Idioma::Hungaro, "operation_done") => "A művelet befejeződött.",
        (Idioma::Hungaro, "operation_failed") => "A művelet sikertelen.",
        (Idioma::Hungaro, "confirm_operation") => "Művelet megerősítése",
        (Idioma::Hungaro, "current") => "Jelenlegi",
        (Idioma::Hungaro, "host") => "Számítógép",
        (Idioma::Hungaro, "model") => "Modell",
        (Idioma::Hungaro, "cpu") => "Processzor",
        (Idioma::Hungaro, "memory") => "Memória",
        (Idioma::Hungaro, "boot") => "Rendszerindítás típusa",
        (Idioma::Hungaro, "applied") => "A módosítás alkalmazva.",
        (Idioma::Hungaro, "change_failed") => "A módosítást nem sikerült alkalmazni.",
        (Idioma::Hungaro, "language") => "Nyelv",
        (Idioma::Hungaro, "region") => "Régió",
        (Idioma::Hungaro, "timezone") => "Időzóna",
        (Idioma::Hungaro, "keyboard") => "Billentyűzet",
        (Idioma::Hungaro, "status") => "Állapot",
        (Idioma::Hungaro, "loading") => "A számítógép adatainak olvasása…",
        (Idioma::Hungaro, "ready") => "Kész",
        (Idioma::Hungaro, "error") => "A Korunix nem tudta beolvasni ezt a területet.",
        (Idioma::Hungaro, "empty") => "Nincs elérhető információ.",

        (_, "subtitle") => "Centro de control de NixOS",
        (_, "summary") => "Resumen",
        (_, "updates") => "Actualizaciones",
        (_, "localization") => "Idioma y región",
        (_, "hardware") => "Hardware",
        (_, "people") => "Personas",
        (_, "refresh") => "Actualizar",
        (_, "channel") => "Canal del sistema",
        (_, "stable") => "Estable",
        (_, "unstable") => "Inestable",
        (_, "prepare") => "Preparar cambio",
        (_, "configured") => "Configurado",
        (_, "target_channel") => "Nuevo canal",
        (_, "change") => "Cambio",
        (_, "confirm_channel") => "Confirmar cambio de canal",
        (_, "cancel") => "Cancelar",
        (_, "apply_change") => "Aplicar cambio",
        (_, "no_change") => "Elige primero un canal distinto.",
        (_, "plan_failed") => "No se pudo preparar la previsualización del cambio.",
        (_, "sections") => "Secciones",
        (_, "media") => "Sonido y cámara",
        (_, "storage") => "Almacenamiento",
        (_, "firmware_updates") => "Firmware",
        (_, "maintenance") => "Mantenimiento",
        (_, "recovery") => "Versiones para recuperación",
        (_, "cleanup") => "Limpieza",
        (_, "normal_cleanup") => "Limpieza recomendada",
        (_, "deep_cleanup") => "Limpieza profunda",
        (_, "clean_now") => "Limpiar",
        (_, "clean_all") => "Limpiar más",
        (_, "generations") => "Versiones disponibles",
        (_, "use_once") => "Probar en el próximo reinicio",
        (_, "current_generation") => "Versión que estás usando",
        (_, "default_generation") => "Versión que inicia normalmente",
        (_, "heavy_transfer") => "Esperar a que terminen de guardarse los datos",
        (_, "heavy_transfer_detail") => {
            "Antes de expulsar, espera a que todos los datos terminen de guardarse en la unidad."
        }
        (_, "eject") => "Expulsar",
        (_, "removable") => "Extraíble",
        (_, "internal") => "Interno",
        (_, "safe_disconnect") => "Ya puedes desconectarla con seguridad.",
        (_, "available_updates") => "Actualizaciones de firmware",
        (_, "firmware_devices") => "Dispositivos de firmware",
        (_, "refresh_firmware") => "Comprobar actualizaciones",
        (_, "install") => "Instalar",
        (_, "no_updates") => "No hay actualizaciones disponibles.",
        (_, "version") => "Versión",
        (_, "output") => "Salida de sonido",
        (_, "input") => "Micrófono",
        (_, "volume") => "Volumen",
        (_, "mute") => "Silenciar",
        (_, "cameras") => "Cámaras",
        (_, "software_sources") => "Catálogo de software",
        (_, "update_sources") => "Actualizar catálogo",
        (_, "privileges") => "Autorización",
        (_, "operation_done") => "Operación completada.",
        (_, "operation_failed") => "La operación falló.",
        (_, "confirm_operation") => "Confirmar operación",
        (_, "current") => "Actual",
        (_, "host") => "Equipo",
        (_, "model") => "Modelo",
        (_, "cpu") => "Procesador",
        (_, "memory") => "Memoria",
        (_, "boot") => "Tipo de arranque",
        (_, "applied") => "Cambio aplicado.",
        (_, "change_failed") => "No se pudo aplicar el cambio.",
        (_, "language") => "Idioma",
        (_, "region") => "Región",
        (_, "timezone") => "Zona horaria",
        (_, "keyboard") => "Teclado",
        (_, "status") => "Estado",
        (_, "loading") => "Leyendo este equipo…",
        (_, "ready") => "Listo",
        (_, "error") => "Korunix no pudo leer esta área.",
        (_, "empty") => "No hay información disponible.",

        (Idioma::Ingles, "applications") => "Applications",
        (Idioma::BelarusLatino, "applications") => "Prykładanni",
        (Idioma::Belarus, "applications") => "Праграмы",
        (Idioma::Catalan, "applications") => "Aplicacions",
        (Idioma::Checo, "applications") => "Aplikace",
        (Idioma::Aleman, "applications") => "Anwendungen",
        (Idioma::Frances, "applications") => "Applications",
        (Idioma::Gallego, "applications") => "Aplicacións",
        (Idioma::Italiano, "applications") => "Applicazioni",
        (Idioma::Coreano, "applications") => "앱",
        (Idioma::Kurdo, "applications") => "Sepan...",
        (Idioma::Neerlandes, "applications") => "Applicaties",
        (Idioma::NoruegoNynorsk, "applications") => "Applikasjonar",
        (Idioma::Polaco, "applications") => "Aplikacje",
        (Idioma::PortuguesBrasil, "applications") => "Aplicações",
        (Idioma::Ruso, "applications") => "Приложения",
        (Idioma::Sueco, "applications") => "Applikationer",
        (Idioma::Turco, "applications") => "Uygulamalar",
        (Idioma::Ucraniano, "applications") => "Застосунки",
        (Idioma::Vietnamita, "applications") => "Ứng dụng",
        (Idioma::ChinoSimplificado, "applications") => "应用程序",
        (Idioma::Hungaro, "applications") => "Alkalmazások",
        (_, "applications") => "Aplicaciones",
        (Idioma::Ingles, "appearance_desktops") => "Appearance and desktops",
        (Idioma::BelarusLatino, "appearance_desktops") => "Znješni vyhłjad i pracoŭny stoł",
        (Idioma::Belarus, "appearance_desktops") => "Знешні выгляд і працоўныя сталы",
        (Idioma::Catalan, "appearance_desktops") => "Aparença i escriptoris",
        (Idioma::Checo, "appearance_desktops") => "Vzhled a stolní počítače",
        (Idioma::Aleman, "appearance_desktops") => "Aussehen und Desktops",
        (Idioma::Frances, "appearance_desktops") => "Apparence et bureaux",
        (Idioma::Gallego, "appearance_desktops") => "Aparencia e escritorios",
        (Idioma::Italiano, "appearance_desktops") => "Aspetto e desktop",
        (Idioma::Coreano, "appearance_desktops") => "모양 및 데스크탑",
        (Idioma::Kurdo, "appearance_desktops") => "Xuyabûn û sermaseyên",
        (Idioma::Neerlandes, "appearance_desktops") => "Uiterlijk en desktops",
        (Idioma::NoruegoNynorsk, "appearance_desktops") => "Utseende og skrivebord",
        (Idioma::Polaco, "appearance_desktops") => "Wygląd i komputery stacjonarne",
        (Idioma::PortuguesBrasil, "appearance_desktops") => "Aparência e áreas de trabalho",
        (Idioma::Ruso, "appearance_desktops") => "Внешний вид и рабочие столы",
        (Idioma::Sueco, "appearance_desktops") => "Utseende och skrivbord",
        (Idioma::Turco, "appearance_desktops") => "Appearance and desktops",
        (Idioma::Ucraniano, "appearance_desktops") => "Зовнішній вигляд і робочі столи",
        (Idioma::Vietnamita, "appearance_desktops") => "Ngoại hình và máy tính để bàn",
        (Idioma::ChinoSimplificado, "appearance_desktops") => "外观和桌面",
        (Idioma::Hungaro, "appearance_desktops") => "Megjelenés és asztalok",
        (_, "appearance_desktops") => "Apariencia y escritorios",
        (Idioma::Ingles, "backups_history") => "Backups and history",
        (Idioma::BelarusLatino, "backups_history") => "Rezjervovyja kopii i historyja",
        (Idioma::Belarus, "backups_history") => "Рэзервовыя копіі і гісторыя",
        (Idioma::Catalan, "backups_history") => "Còpies de seguretat i historial",
        (Idioma::Checo, "backups_history") => "Zálohy a historie",
        (Idioma::Aleman, "backups_history") => "Backups und Verlauf",
        (Idioma::Frances, "backups_history") => "Sauvegardes et historique",
        (Idioma::Gallego, "backups_history") => "Copias de seguridade e historial",
        (Idioma::Italiano, "backups_history") => "Backup e cronologia",
        (Idioma::Coreano, "backups_history") => "백업 및 기록",
        (Idioma::Kurdo, "backups_history") => "Backup û dîrok",
        (Idioma::Neerlandes, "backups_history") => "Back-ups en geschiedenis",
        (Idioma::NoruegoNynorsk, "backups_history") => "Sikkerhetskopier og historikk",
        (Idioma::Polaco, "backups_history") => "Kopie zapasowe i historia",
        (Idioma::PortuguesBrasil, "backups_history") => "Backups e histórico",
        (Idioma::Ruso, "backups_history") => "Резервные копии и история",
        (Idioma::Sueco, "backups_history") => "Säkerhetskopiering och historik",
        (Idioma::Turco, "backups_history") => "Backups and history",
        (Idioma::Ucraniano, "backups_history") => "Резервні копії та історія",
        (Idioma::Vietnamita, "backups_history") => "Sao lưu và lịch sử",
        (Idioma::ChinoSimplificado, "backups_history") => "备份和历史记录",
        (Idioma::Hungaro, "backups_history") => "Biztonsági mentés és előzmények",
        (_, "backups_history") => "Copias e historial",
        (Idioma::Ingles, "global_search") => "Search Korunix",
        (Idioma::BelarusLatino, "global_search") => "Pošuk u Korunix",
        (Idioma::Belarus, "global_search") => "Пошук у Korunix",
        (Idioma::Catalan, "global_search") => "Cerca Korunix",
        (Idioma::Checo, "global_search") => "Prohledejte Korunix",
        (Idioma::Aleman, "global_search") => "Korunix suchen",
        (Idioma::Frances, "global_search") => "Rechercher Korunix",
        (Idioma::Gallego, "global_search") => "Busca Korunix",
        (Idioma::Italiano, "global_search") => "Cerca Korunix",
        (Idioma::Coreano, "global_search") => "Korunix 검색",
        (Idioma::Kurdo, "global_search") => "Lêgerîna Korunix",
        (Idioma::Neerlandes, "global_search") => "Zoek Korunix",
        (Idioma::NoruegoNynorsk, "global_search") => "Søk i Korunix",
        (Idioma::Polaco, "global_search") => "Wyszukaj Korunix",
        (Idioma::PortuguesBrasil, "global_search") => "Pesquisa Korunix",
        (Idioma::Ruso, "global_search") => "Поиск Korunix",
        (Idioma::Sueco, "global_search") => "Sök Korunix",
        (Idioma::Turco, "global_search") => "Korunix'te arama yapın",
        (Idioma::Ucraniano, "global_search") => "Пошук у Korunix",
        (Idioma::Vietnamita, "global_search") => "Tìm kiếm Korunix",
        (Idioma::ChinoSimplificado, "global_search") => "搜索 Korunix",
        (Idioma::Hungaro, "global_search") => "Keresés a Korunixban",
        (_, "global_search") => "Buscar en Korunix",
        (Idioma::Ingles, "save_apply") => "Save and apply",
        (Idioma::BelarusLatino, "save_apply") => "Zachavajcje i prymjanicje",
        (Idioma::Belarus, "save_apply") => "Захаваць і прымяніць",
        (Idioma::Catalan, "save_apply") => "Desa i aplica",
        (Idioma::Checo, "save_apply") => "Uložit a použít",
        (Idioma::Aleman, "save_apply") => "Speichern und anwenden",
        (Idioma::Frances, "save_apply") => "Enregistrer et appliquer",
        (Idioma::Gallego, "save_apply") => "Garda e aplica",
        (Idioma::Italiano, "save_apply") => "Salva e applica",
        (Idioma::Coreano, "save_apply") => "저장 및 적용",
        (Idioma::Kurdo, "save_apply") => "Hilînin û bicîh bikin",
        (Idioma::Neerlandes, "save_apply") => "Opslaan en toepassen",
        (Idioma::NoruegoNynorsk, "save_apply") => "Lagre og bruk",
        (Idioma::Polaco, "save_apply") => "Zapisz i zastosuj",
        (Idioma::PortuguesBrasil, "save_apply") => "Salvar e aplicar",
        (Idioma::Ruso, "save_apply") => "Сохранить и применить",
        (Idioma::Sueco, "save_apply") => "Spara och tillämpa",
        (Idioma::Turco, "save_apply") => "Kaydet ve uygula",
        (Idioma::Ucraniano, "save_apply") => "Зберегти та застосувати",
        (Idioma::Vietnamita, "save_apply") => "Lưu và áp dụng",
        (Idioma::ChinoSimplificado, "save_apply") => "保存并应用",
        (Idioma::Hungaro, "save_apply") => "Mentés és alkalmazás",
        (_, "save_apply") => "Guardar y aplicar",
        (Idioma::Ingles, "remove") => "Remove",
        (Idioma::BelarusLatino, "remove") => "Znjac",
        (Idioma::Belarus, "remove") => "Выдаліць",
        (Idioma::Catalan, "remove") => "Elimina",
        (Idioma::Checo, "remove") => "Odebrat",
        (Idioma::Aleman, "remove") => "Entfernen",
        (Idioma::Frances, "remove") => "Supprimer",
        (Idioma::Gallego, "remove") => "Eliminar",
        (Idioma::Italiano, "remove") => "Rimuovi",
        (Idioma::Coreano, "remove") => "제거",
        (Idioma::Kurdo, "remove") => "Rake",
        (Idioma::Neerlandes, "remove") => "Verwijderen",
        (Idioma::NoruegoNynorsk, "remove") => "Fjern",
        (Idioma::Polaco, "remove") => "Usuń",
        (Idioma::PortuguesBrasil, "remove") => "Remover",
        (Idioma::Ruso, "remove") => "Удалить",
        (Idioma::Sueco, "remove") => "Ta bort",
        (Idioma::Turco, "remove") => "Kaldır",
        (Idioma::Ucraniano, "remove") => "Видалити",
        (Idioma::Vietnamita, "remove") => "Xóa",
        (Idioma::ChinoSimplificado, "remove") => "删除",
        (Idioma::Hungaro, "remove") => "Eltávolítás",
        (_, "remove") => "Eliminar",
        (Idioma::Ingles, "search") => "Search",
        (Idioma::BelarusLatino, "search") => "Pošuk",
        (Idioma::Belarus, "search") => "Пошук",
        (Idioma::Catalan, "search") => "Cerca",
        (Idioma::Checo, "search") => "Hledat",
        (Idioma::Aleman, "search") => "Suche",
        (Idioma::Frances, "search") => "Recherche",
        (Idioma::Gallego, "search") => "Busca",
        (Idioma::Italiano, "search") => "Cerca",
        (Idioma::Coreano, "search") => "검색",
        (Idioma::Kurdo, "search") => "Lêgerîn",
        (Idioma::Neerlandes, "search") => "Zoeken",
        (Idioma::NoruegoNynorsk, "search") => "Søk",
        (Idioma::Polaco, "search") => "Szukaj",
        (Idioma::PortuguesBrasil, "search") => "Pesquisa",
        (Idioma::Ruso, "search") => "Поиск",
        (Idioma::Sueco, "search") => "Sök",
        (Idioma::Turco, "search") => "Ara",
        (Idioma::Ucraniano, "search") => "Пошук",
        (Idioma::Vietnamita, "search") => "Tìm kiếm",
        (Idioma::ChinoSimplificado, "search") => "搜索",
        (Idioma::Hungaro, "search") => "Keresés",
        (_, "search") => "Buscar",
        (Idioma::Ingles, "export_backup") => "Export backup",
        (Idioma::BelarusLatino, "export_backup") => "Ekspart rezjervovaj kopii",
        (Idioma::Belarus, "export_backup") => "Экспарт рэзервовай копіі",
        (Idioma::Catalan, "export_backup") => "Exporta la còpia de seguretat",
        (Idioma::Checo, "export_backup") => "Exportujte zálohu",
        (Idioma::Aleman, "export_backup") => "Sicherung exportieren",
        (Idioma::Frances, "export_backup") => "Exporter la sauvegarde",
        (Idioma::Gallego, "export_backup") => "Exportar copia de seguranza",
        (Idioma::Italiano, "export_backup") => "Esporta backup",
        (Idioma::Coreano, "export_backup") => "백업 내보내기",
        (Idioma::Kurdo, "export_backup") => "Piştgiriya hinardekirinê",
        (Idioma::Neerlandes, "export_backup") => "Back-up exporteren",
        (Idioma::NoruegoNynorsk, "export_backup") => "Eksporter sikkerhetskopi",
        (Idioma::Polaco, "export_backup") => "Eksportuj kopię zapasową",
        (Idioma::PortuguesBrasil, "export_backup") => "Exportar backup",
        (Idioma::Ruso, "export_backup") => "Экспортировать резервную копию",
        (Idioma::Sueco, "export_backup") => "Exportera säkerhetskopia",
        (Idioma::Turco, "export_backup") => "Export backup",
        (Idioma::Ucraniano, "export_backup") => "Експорт резервної копії",
        (Idioma::Vietnamita, "export_backup") => "Xuất bản sao lưu",
        (Idioma::ChinoSimplificado, "export_backup") => "导出备份",
        (Idioma::Hungaro, "export_backup") => "Biztonsági mentés exportálása",
        (_, "export_backup") => "Exportar copia",
        (Idioma::Ingles, "restore_backup") => "Restore backup",
        (Idioma::BelarusLatino, "restore_backup") => "Adnavic rezjervovuju kopiju",
        (Idioma::Belarus, "restore_backup") => "Аднавіць рэзервовую копію",
        (Idioma::Catalan, "restore_backup") => "Restaura la còpia de seguretat",
        (Idioma::Checo, "restore_backup") => "Obnovte zálohu",
        (Idioma::Aleman, "restore_backup") => "Sicherung wiederherstellen",
        (Idioma::Frances, "restore_backup") => "Restaurer la sauvegarde",
        (Idioma::Gallego, "restore_backup") => "Restaurar copia de seguranza",
        (Idioma::Italiano, "restore_backup") => "Ripristina backup",
        (Idioma::Coreano, "restore_backup") => "백업 복원",
        (Idioma::Kurdo, "restore_backup") => "Piştgiriyê vegerîne",
        (Idioma::Neerlandes, "restore_backup") => "Back-up herstellen",
        (Idioma::NoruegoNynorsk, "restore_backup") => "Gjenopprett sikkerhetskopi",
        (Idioma::Polaco, "restore_backup") => "Przywróć kopię zapasową",
        (Idioma::PortuguesBrasil, "restore_backup") => "Restaurar backup",
        (Idioma::Ruso, "restore_backup") => "Восстановить резервную копию",
        (Idioma::Sueco, "restore_backup") => "Återställ säkerhetskopia",
        (Idioma::Turco, "restore_backup") => "Yedeği geri yükle",
        (Idioma::Ucraniano, "restore_backup") => "Відновити резервну копію",
        (Idioma::Vietnamita, "restore_backup") => "Khôi phục bản sao lưu",
        (Idioma::ChinoSimplificado, "restore_backup") => "恢复备份",
        (Idioma::Hungaro, "restore_backup") => "Biztonsági mentés visszaállítása",
        (_, "restore_backup") => "Restaurar copia",
        (Idioma::Ingles, "create_person") => "Create person",
        (Idioma::BelarusLatino, "create_person") => "Stvaryc čałavjeka",
        (Idioma::Belarus, "create_person") => "Стварыць чалавека",
        (Idioma::Catalan, "create_person") => "Crea una persona",
        (Idioma::Checo, "create_person") => "Vytvořit osobu",
        (Idioma::Aleman, "create_person") => "Person erstellen",
        (Idioma::Frances, "create_person") => "Créer une personne",
        (Idioma::Gallego, "create_person") => "Crear persoa",
        (Idioma::Italiano, "create_person") => "Crea persona",
        (Idioma::Coreano, "create_person") => "사람 만들기",
        (Idioma::Kurdo, "create_person") => "Mirov biafirîne",
        (Idioma::Neerlandes, "create_person") => "Persoon aanmaken",
        (Idioma::NoruegoNynorsk, "create_person") => "Opprett person",
        (Idioma::Polaco, "create_person") => "Utwórz osobę",
        (Idioma::PortuguesBrasil, "create_person") => "Criar pessoa",
        (Idioma::Ruso, "create_person") => "Создать человека",
        (Idioma::Sueco, "create_person") => "Skapa person",
        (Idioma::Turco, "create_person") => "Create person",
        (Idioma::Ucraniano, "create_person") => "Створити людину",
        (Idioma::Vietnamita, "create_person") => "Tạo người",
        (Idioma::ChinoSimplificado, "create_person") => "创建人",
        (Idioma::Hungaro, "create_person") => "Személy létrehozása",
        (_, "create_person") => "Crear persona",
        (Idioma::Ingles, "update_all") => "Update all",
        (Idioma::BelarusLatino, "update_all") => "Abnavic usjo",
        (Idioma::Belarus, "update_all") => "Абнавіць усё",
        (Idioma::Catalan, "update_all") => "Actualitza-ho tot",
        (Idioma::Checo, "update_all") => "Aktualizovat vše",
        (Idioma::Aleman, "update_all") => "Alle aktualisieren",
        (Idioma::Frances, "update_all") => "Tout mettre à jour",
        (Idioma::Gallego, "update_all") => "Actualizar todo",
        (Idioma::Italiano, "update_all") => "Aggiorna tutto",
        (Idioma::Coreano, "update_all") => "모두 업데이트",
        (Idioma::Kurdo, "update_all") => "Hemî nûve bikin",
        (Idioma::Neerlandes, "update_all") => "Alles bijwerken",
        (Idioma::NoruegoNynorsk, "update_all") => "Oppdater alle",
        (Idioma::Polaco, "update_all") => "Zaktualizuj wszystko",
        (Idioma::PortuguesBrasil, "update_all") => "Atualizar tudo",
        (Idioma::Ruso, "update_all") => "Обновить все",
        (Idioma::Sueco, "update_all") => "Uppdatera alla",
        (Idioma::Turco, "update_all") => "Tümünü güncelle",
        (Idioma::Ucraniano, "update_all") => "Оновити все",
        (Idioma::Vietnamita, "update_all") => "Cập nhật tất cả",
        (Idioma::ChinoSimplificado, "update_all") => "全部更新",
        (Idioma::Hungaro, "update_all") => "Összes frissítése",
        (_, "update_all") => "Actualizar todo",
        (Idioma::Ingles, "customize_updates") => "Customize",
        (Idioma::BelarusLatino, "customize_updates") => "Naładzic",
        (Idioma::Belarus, "customize_updates") => "Дапасаваць",
        (Idioma::Catalan, "customize_updates") => "Personalitza",
        (Idioma::Checo, "customize_updates") => "Přizpůsobit",
        (Idioma::Aleman, "customize_updates") => "Anpassen",
        (Idioma::Frances, "customize_updates") => "Personnaliser",
        (Idioma::Gallego, "customize_updates") => "Personalizar",
        (Idioma::Italiano, "customize_updates") => "Personalizza",
        (Idioma::Coreano, "customize_updates") => "사용자 정의",
        (Idioma::Kurdo, "customize_updates") => "Kesane bike",
        (Idioma::Neerlandes, "customize_updates") => "Aanpassen",
        (Idioma::NoruegoNynorsk, "customize_updates") => "Måta til",
        (Idioma::Polaco, "customize_updates") => "Dostosuj",
        (Idioma::PortuguesBrasil, "customize_updates") => "Customizar",
        (Idioma::Ruso, "customize_updates") => "Настроить",
        (Idioma::Sueco, "customize_updates") => "Anpassa",
        (Idioma::Turco, "customize_updates") => "Özelleştir",
        (Idioma::Ucraniano, "customize_updates") => "Налаштувати",
        (Idioma::Vietnamita, "customize_updates") => "Tùy chỉnh",
        (Idioma::ChinoSimplificado, "customize_updates") => "自定义",
        (Idioma::Hungaro, "customize_updates") => "Testreszabás",
        (_, "customize_updates") => "Personalizar",
        (Idioma::Ingles, "advanced") => "Advanced",
        (Idioma::BelarusLatino, "advanced") => "Pašyrany",
        (Idioma::Belarus, "advanced") => "Пашыраныя",
        (Idioma::Catalan, "advanced") => "Avançat",
        (Idioma::Checo, "advanced") => "Pokročilé",
        (Idioma::Aleman, "advanced") => "Erweitert",
        (Idioma::Frances, "advanced") => "Avancé",
        (Idioma::Gallego, "advanced") => "Avanzado",
        (Idioma::Italiano, "advanced") => "Avanzato",
        (Idioma::Coreano, "advanced") => "고급",
        (Idioma::Kurdo, "advanced") => "Pêşketî",
        (Idioma::Neerlandes, "advanced") => "Geavanceerd",
        (Idioma::NoruegoNynorsk, "advanced") => "Avansert",
        (Idioma::Polaco, "advanced") => "Zaawansowane",
        (Idioma::PortuguesBrasil, "advanced") => "Avançado",
        (Idioma::Ruso, "advanced") => "Расширенное",
        (Idioma::Sueco, "advanced") => "Avancerat",
        (Idioma::Turco, "advanced") => "Gelişmiş",
        (Idioma::Ucraniano, "advanced") => "Додаткові",
        (Idioma::Vietnamita, "advanced") => "Nâng cao",
        (Idioma::ChinoSimplificado, "advanced") => "高级",
        (Idioma::Hungaro, "advanced") => "Haladó",
        (_, "advanced") => "Avanzado",
        _ => "Korunix",
    }
}

fn raiz_proyecto() -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_ROOT") {
        let ruta = PathBuf::from(valor);
        if ruta.join("flake.nix").is_file() {
            return Ok(ruta);
        }
    }

    if let Ok(actual) = env::current_dir() {
        if actual.join("flake.nix").is_file() {
            return Ok(actual);
        }
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME no está disponible.".to_string())?;

    let ruta = home.join(".korunix");

    if ruta.join("flake.nix").is_file() {
        Ok(ruta)
    } else {
        Err("No encuentro el checkout de Korunix.".to_string())
    }
}

fn motor(raiz: &Path) -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_MOTOR_BIN") {
        let ruta = PathBuf::from(valor);
        if ruta.is_file() {
            return Ok(ruta);
        }
    }

    let desarrollo = raiz.join("target/debug/korunix");

    if desarrollo.is_file() {
        return Ok(desarrollo);
    }

    Err("No encuentro el motor Rust de Korunix.".to_string())
}

struct Estado {
    raiz: PathBuf,
    motor: PathBuf,
    idioma: Idioma,
    stack: gtk::Stack,
    navegacion: gtk::ListBox,
    pagina_contenido: adw::NavigationPage,
    toast: adw::ToastOverlay,
    progreso: gtk::Revealer,
    progreso_barra: gtk::ProgressBar,
    progreso_texto: gtk::Label,
    cargando: Cell<bool>,
    ocupado: Cell<bool>,
    camara_preview_activa: Cell<bool>,
    _apariencia: AparienciaViva,
}

enum EventoMotor {
    Progreso(u8, String),
    Terminado(Result<String, String>),
}

fn texto_progreso(idioma: Idioma, clave: &str) -> &'static str {
    match (idioma, clave) {
        (Idioma::Ingles, "reading") => "Reading this computer…",
        (Idioma::BelarusLatino, "reading") => "Čytannje hetaha kamp'jutara...",
        (Idioma::Belarus, "reading") => "Чытанне гэтага кампутара...",
        (Idioma::Catalan, "reading") => "Llegint aquest ordinador...",
        (Idioma::Checo, "reading") => "Čtení tohoto počítače…",
        (Idioma::Aleman, "reading") => "Dieser Computer wird gelesen…",
        (Idioma::Frances, "reading") => "Lecture de cet ordinateur…",
        (Idioma::Gallego, "reading") => "Lendo este ordenador…",
        (Idioma::Italiano, "reading") => "Lettura di questo computer…",
        (Idioma::Coreano, "reading") => "이 컴퓨터를 읽는 중…",
        (Idioma::Kurdo, "reading") => "Xwendina vê kompîturê…",
        (Idioma::Neerlandes, "reading") => "Deze computer lezen…",
        (Idioma::NoruegoNynorsk, "reading") => "Leser denne datamaskinen...",
        (Idioma::Polaco, "reading") => "Czytanie tego komputera…",
        (Idioma::PortuguesBrasil, "reading") => "Lendo este computador…",
        (Idioma::Ruso, "reading") => "Читаю этот компьютер…",
        (Idioma::Sueco, "reading") => "Läser den här datorn...",
        (Idioma::Turco, "reading") => "Bu bilgisayar okunuyor…",
        (Idioma::Ucraniano, "reading") => "Читання цього комп’ютера…",
        (Idioma::Vietnamita, "reading") => "Đang đọc máy tính này…",
        (Idioma::ChinoSimplificado, "reading") => "正在读取这台计算机...",
        (Idioma::Ingles, "preparing") => "Preparing…",
        (Idioma::BelarusLatino, "preparing") => "Padrychtoŭka…",
        (Idioma::Belarus, "preparing") => "Падрыхтоўка…",
        (Idioma::Catalan, "preparing") => "S'està preparant…",
        (Idioma::Checo, "preparing") => "Příprava…",
        (Idioma::Aleman, "preparing") => "Vorbereiten…",
        (Idioma::Frances, "preparing") => "Préparation…",
        (Idioma::Gallego, "preparing") => "Preparando…",
        (Idioma::Italiano, "preparing") => "Preparazione...",
        (Idioma::Coreano, "preparing") => "준비 중…",
        (Idioma::Kurdo, "preparing") => "Amadekirin…",
        (Idioma::Neerlandes, "preparing") => "Voorbereiden…",
        (Idioma::NoruegoNynorsk, "preparing") => "Forbereder...",
        (Idioma::Polaco, "preparing") => "Przygotowywanie…",
        (Idioma::PortuguesBrasil, "preparing") => "Preparando…",
        (Idioma::Ruso, "preparing") => "Подготовка…",
        (Idioma::Sueco, "preparing") => "Förbereder...",
        (Idioma::Turco, "preparing") => "Hazırlanıyor…",
        (Idioma::Ucraniano, "preparing") => "Підготовка…",
        (Idioma::Vietnamita, "preparing") => "Đang chuẩn bị…",
        (Idioma::ChinoSimplificado, "preparing") => "准备...",
        (Idioma::Ingles, "changing_channel") => "Changing the system channel…",
        (Idioma::BelarusLatino, "changing_channel") => "Zmjena sistemnaha kanała…",
        (Idioma::Belarus, "changing_channel") => "Змена сістэмнага канала…",
        (Idioma::Catalan, "changing_channel") => "Canviant el canal del sistema...",
        (Idioma::Checo, "changing_channel") => "Změna systémového kanálu…",
        (Idioma::Aleman, "changing_channel") => "Ändern des Systemkanals…",
        (Idioma::Frances, "changing_channel") => "Modification du canal système…",
        (Idioma::Gallego, "changing_channel") => "Cambiando a canle do sistema...",
        (Idioma::Italiano, "changing_channel") => "Modifica del canale del sistema…",
        (Idioma::Coreano, "changing_channel") => "시스템 채널 변경 중...",
        (Idioma::Kurdo, "changing_channel") => "Guhertina kanala pergalê…",
        (Idioma::Neerlandes, "changing_channel") => "Systeemkanaal wijzigen…",
        (Idioma::NoruegoNynorsk, "changing_channel") => "Endre systemkanalen...",
        (Idioma::Polaco, "changing_channel") => "Zmiana kanału systemowego…",
        (Idioma::PortuguesBrasil, "changing_channel") => "Alterando o canal do sistema…",
        (Idioma::Ruso, "changing_channel") => "Изменение системного канала…",
        (Idioma::Sueco, "changing_channel") => "Ändra systemkanalen...",
        (Idioma::Turco, "changing_channel") => "Changing the system channel…",
        (Idioma::Ucraniano, "changing_channel") => "Зміна системного каналу…",
        (Idioma::Vietnamita, "changing_channel") => "Thay đổi kênh hệ thống…",
        (Idioma::ChinoSimplificado, "changing_channel") => "更改系统频道...",
        (Idioma::Ingles, "updating_catalog") => "Updating the software catalog…",
        (Idioma::BelarusLatino, "updating_catalog") => {
            "Abnaŭłjennje katałoha prahramnaha zabjespjačennja…"
        }
        (Idioma::Belarus, "updating_catalog") => "Абнаўленне каталога праграмнага забеспячэння…",
        (Idioma::Catalan, "updating_catalog") => "S'està actualitzant el catàleg de programari...",
        (Idioma::Checo, "updating_catalog") => "Aktualizace katalogu softwaru…\nVerze __KX00129__",
        (Idioma::Aleman, "updating_catalog") => "Aktualisierung des Softwarekatalogs…",
        (Idioma::Frances, "updating_catalog") => {
            "Mise à jour du catalogue de logiciels…\nVersion __KX00129__"
        }
        (Idioma::Gallego, "updating_catalog") => "Actualizando o catálogo de software...",
        (Idioma::Italiano, "updating_catalog") => "Aggiornamento del catalogo software…",
        (Idioma::Coreano, "updating_catalog") => "소프트웨어 카탈로그 업데이트 중…",
        (Idioma::Kurdo, "updating_catalog") => "Nûvekirina kataloga nermalavê…",
        (Idioma::Neerlandes, "updating_catalog") => "De softwarecatalogus bijwerken…",
        (Idioma::NoruegoNynorsk, "updating_catalog") => "Oppdaterer programvarekatalogen...",
        (Idioma::Polaco, "updating_catalog") => "Aktualizowanie katalogu oprogramowania…",
        (Idioma::PortuguesBrasil, "updating_catalog") => "Atualizando o catálogo de software…",
        (Idioma::Ruso, "updating_catalog") => "Обновление каталога программного обеспечения…",
        (Idioma::Sueco, "updating_catalog") => "Uppdaterar programvarukatalogen...",
        (Idioma::Turco, "updating_catalog") => "Yazılım kataloğu güncelleniyor…",
        (Idioma::Ucraniano, "updating_catalog") => "Оновлення каталогу програмного забезпечення…",
        (Idioma::Vietnamita, "updating_catalog") => "Cập nhật danh mục phần mềm…",
        (Idioma::ChinoSimplificado, "updating_catalog") => "正在更新软件目录...",
        (Idioma::Ingles, "validating") => "Checking the updated configuration…",
        (Idioma::BelarusLatino, "validating") => "Pravjerka abnoŭłjenaj kanfihuracyi…",
        (Idioma::Belarus, "validating") => "Праверка абноўленай канфігурацыі…",
        (Idioma::Catalan, "validating") => "S'està comprovant la configuració actualitzada...",
        (Idioma::Checo, "validating") => "Kontrola aktualizované konfigurace…",
        (Idioma::Aleman, "validating") => "Überprüfen der aktualisierten Konfiguration…",
        (Idioma::Frances, "validating") => "Vérification de la configuration mise à jour…",
        (Idioma::Gallego, "validating") => "Comprobando a configuración actualizada...",
        (Idioma::Italiano, "validating") => "Controllo della configurazione aggiornata…",
        (Idioma::Coreano, "validating") => "업데이트된 구성을 확인하는 중…",
        (Idioma::Kurdo, "validating") => "Veavakirina nûvekirî kontrol dike…",
        (Idioma::Neerlandes, "validating") => "De bijgewerkte configuratie controleren…",
        (Idioma::NoruegoNynorsk, "validating") => "Kontrollerer den oppdaterte konfigurasjonen...",
        (Idioma::Polaco, "validating") => "Sprawdzam zaktualizowaną konfigurację…",
        (Idioma::PortuguesBrasil, "validating") => "Verificando a configuração atualizada…",
        (Idioma::Ruso, "validating") => "Проверка обновленной конфигурации…",
        (Idioma::Sueco, "validating") => "Kontrollerar den uppdaterade konfigurationen...",
        (Idioma::Turco, "validating") => "Checking the updated configuration…",
        (Idioma::Ucraniano, "validating") => "Перевірка оновленої конфігурації…",
        (Idioma::Vietnamita, "validating") => "Đang kiểm tra cấu hình cập nhật…",
        (Idioma::ChinoSimplificado, "validating") => "检查更新的配置...",
        (Idioma::Ingles, "cleaning_versions") => "Removing old system versions…",
        (Idioma::BelarusLatino, "cleaning_versions") => "Vydałjennje starych vjersij sistemy…",
        (Idioma::Belarus, "cleaning_versions") => "Выдаленне старых версій сістэмы…",
        (Idioma::Catalan, "cleaning_versions") => {
            "S'estan eliminant les versions antigues del sistema..."
        }
        (Idioma::Checo, "cleaning_versions") => "Odebírání starých verzí systému…",
        (Idioma::Aleman, "cleaning_versions") => "Alte Systemversionen werden entfernt…",
        (Idioma::Frances, "cleaning_versions") => "Suppression des anciennes versions du système…",
        (Idioma::Gallego, "cleaning_versions") => "Eliminando versións antigas do sistema...",
        (Idioma::Italiano, "cleaning_versions") => "Rimozione delle vecchie versioni del sistema…",
        (Idioma::Coreano, "cleaning_versions") => "이전 시스템 버전을 제거하는 중…",
        (Idioma::Kurdo, "cleaning_versions") => "Rakirina guhertoyên pergalê yên kevn…",
        (Idioma::Neerlandes, "cleaning_versions") => "Oude systeemversies verwijderen…",
        (Idioma::NoruegoNynorsk, "cleaning_versions") => "Fjerner gamle systemversjoner...",
        (Idioma::Polaco, "cleaning_versions") => "Usuwanie starych wersji systemu…",
        (Idioma::PortuguesBrasil, "cleaning_versions") => "Removendo versões antigas do sistema…",
        (Idioma::Ruso, "cleaning_versions") => "Удаление старых версий системы…",
        (Idioma::Sueco, "cleaning_versions") => "Tar bort gamla systemversioner...",
        (Idioma::Turco, "cleaning_versions") => "Eski sistem sürümleri kaldırılıyor…",
        (Idioma::Ucraniano, "cleaning_versions") => "Видалення старих версій системи…",
        (Idioma::Vietnamita, "cleaning_versions") => "Xóa phiên bản hệ thống cũ…",
        (Idioma::ChinoSimplificado, "cleaning_versions") => "正在删除旧系统版本...",
        (Idioma::Ingles, "garbage_collect") => "Freeing unused space…",
        (Idioma::BelarusLatino, "garbage_collect") => "Vyzvałjennje njavykarystanaj prastory…",
        (Idioma::Belarus, "garbage_collect") => "Вызваленне нявыкарыстанай прасторы…",
        (Idioma::Catalan, "garbage_collect") => "S'està alliberant espai no utilitzat...",
        (Idioma::Checo, "garbage_collect") => "Uvolňuje se nevyužité místo…",
        (Idioma::Aleman, "garbage_collect") => "Ungenutzten Speicherplatz wird freigegeben…",
        (Idioma::Frances, "garbage_collect") => "Libérer de l'espace inutilisé…",
        (Idioma::Gallego, "garbage_collect") => "Liberando espazo non utilizado...",
        (Idioma::Italiano, "garbage_collect") => "Liberazione dello spazio inutilizzato…",
        (Idioma::Coreano, "garbage_collect") => "사용하지 않는 공간을 확보하는 중…",
        (Idioma::Kurdo, "garbage_collect") => "Cihê nebikaranîn azad dike…",
        (Idioma::Neerlandes, "garbage_collect") => "Ongebruikte ruimte vrijmaken...",
        (Idioma::NoruegoNynorsk, "garbage_collect") => "Frigjør ubrukt plass...",
        (Idioma::Polaco, "garbage_collect") => "Zwalniam nieużywane miejsce…",
        (Idioma::PortuguesBrasil, "garbage_collect") => "Liberando espaço não utilizado…",
        (Idioma::Ruso, "garbage_collect") => "Освобождение неиспользуемого пространства…",
        (Idioma::Sueco, "garbage_collect") => "Frigör oanvänt utrymme...",
        (Idioma::Turco, "garbage_collect") => "Kullanılmayan alan boşaltılıyor…",
        (Idioma::Ucraniano, "garbage_collect") => "Звільнення невикористаного простору…",
        (Idioma::Vietnamita, "garbage_collect") => "Giải phóng không gian chưa sử dụng…",
        (Idioma::ChinoSimplificado, "garbage_collect") => "释放未使用的空间...",
        (Idioma::Ingles, "optimising_store") => "Optimising system storage…",
        (Idioma::BelarusLatino, "optimising_store") => "Aptymizacyja sistemnaha schovišča…",
        (Idioma::Belarus, "optimising_store") => "Аптымізацыя сістэмнага сховішча…",
        (Idioma::Catalan, "optimising_store") => {
            "S'està optimitzant l'emmagatzematge del sistema..."
        }
        (Idioma::Checo, "optimising_store") => "Optimalizace systémového úložiště…",
        (Idioma::Aleman, "optimising_store") => "Systemspeicher optimieren…",
        (Idioma::Frances, "optimising_store") => "Optimisation du stockage système…",
        (Idioma::Gallego, "optimising_store") => "Optimizando o almacenamento do sistema...",
        (Idioma::Italiano, "optimising_store") => {
            "Ottimizzazione dello spazio di archiviazione del sistema…"
        }
        (Idioma::Coreano, "optimising_store") => "시스템 스토리지 최적화 중...",
        (Idioma::Kurdo, "optimising_store") => "Optimîzekirina hilanîna pergalê…",
        (Idioma::Neerlandes, "optimising_store") => "Systeemopslag optimaliseren…",
        (Idioma::NoruegoNynorsk, "optimising_store") => "Optimaliserer systemlagring...",
        (Idioma::Polaco, "optimising_store") => "Optymalizacja pamięci systemowej…",
        (Idioma::PortuguesBrasil, "optimising_store") => "Otimizando o armazenamento do sistema…",
        (Idioma::Ruso, "optimising_store") => "Оптимизация системного хранилища…",
        (Idioma::Sueco, "optimising_store") => "Optimerar systemlagring...",
        (Idioma::Turco, "optimising_store") => "Sistem depolama alanı optimize ediliyor…",
        (Idioma::Ucraniano, "optimising_store") => "Оптимізація системного зберігання…",
        (Idioma::Vietnamita, "optimising_store") => "Tối ưu hóa hệ thống lưu trữ…",
        (Idioma::ChinoSimplificado, "optimising_store") => "优化系统存储...",
        (Idioma::Ingles, "saving_data") => "Finishing pending writes…",
        (Idioma::BelarusLatino, "saving_data") => "Zavjaršennje njezavjeršanych zapisaŭ…",
        (Idioma::Belarus, "saving_data") => "Завяршэнне незавершаных запісаў…",
        (Idioma::Catalan, "saving_data") => "S'estan acabant les escriptures pendents...",
        (Idioma::Checo, "saving_data") => "Dokončování čekajících zápisů…",
        (Idioma::Aleman, "saving_data") => "Schreibvorgänge werden abgeschlossen …",
        (Idioma::Frances, "saving_data") => "Fin des écritures en attente…",
        (Idioma::Gallego, "saving_data") => "Rematando as escrituras pendentes...",
        (Idioma::Italiano, "saving_data") => "Completamento delle scritture in sospeso…",
        (Idioma::Coreano, "saving_data") => "보류 중인 쓰기를 완료하는 중…",
        (Idioma::Kurdo, "saving_data") => "Li benda nivîsandinê qediya…",
        (Idioma::Neerlandes, "saving_data") => "Bezig met schrijven in behandeling…",
        (Idioma::NoruegoNynorsk, "saving_data") => "Fullfører ventende skrivinger...",
        (Idioma::Polaco, "saving_data") => "Zakończenie oczekujących zapisów…",
        (Idioma::PortuguesBrasil, "saving_data") => "Concluindo gravações pendentes…",
        (Idioma::Ruso, "saving_data") => "Ожидается завершение записи…",
        (Idioma::Sueco, "saving_data") => "Avslutar väntande skrivningar...",
        (Idioma::Turco, "saving_data") => "Bekleyen yazma işlemleri tamamlanıyor…",
        (Idioma::Ucraniano, "saving_data") => "Завершення незавершених записів…",
        (Idioma::Vietnamita, "saving_data") => "Đang hoàn tất quá trình ghi đang chờ xử lý…",
        (Idioma::ChinoSimplificado, "saving_data") => "正在完成挂起的写入...",
        (Idioma::Ingles, "unmounting") => "Disconnecting filesystems…",
        (Idioma::BelarusLatino, "unmounting") => "Adkłjučennje fajłavych sistem…",
        (Idioma::Belarus, "unmounting") => "Адключэнне файлавых сістэм…",
        (Idioma::Catalan, "unmounting") => "S'estan desconnectant els sistemes de fitxers...",
        (Idioma::Checo, "unmounting") => "Odpojování souborových systémů…",
        (Idioma::Aleman, "unmounting") => "Dateisysteme werden getrennt…",
        (Idioma::Frances, "unmounting") => "Déconnexion des systèmes de fichiers…",
        (Idioma::Gallego, "unmounting") => "Desconectando sistemas de ficheiros...",
        (Idioma::Italiano, "unmounting") => "Disconnessione dei file system…",
        (Idioma::Coreano, "unmounting") => "파일 시스템 연결을 끊는 중…",
        (Idioma::Kurdo, "unmounting") => "Pergalên pelan qut dike…",
        (Idioma::Neerlandes, "unmounting") => "Bestandssystemen loskoppelen…",
        (Idioma::NoruegoNynorsk, "unmounting") => "Kobler fra filsystemer...",
        (Idioma::Polaco, "unmounting") => "Odłączanie systemów plików…",
        (Idioma::PortuguesBrasil, "unmounting") => "Desconectando sistemas de arquivos…",
        (Idioma::Ruso, "unmounting") => "Отключение файловых систем…",
        (Idioma::Sueco, "unmounting") => "Kopplar bort filsystem...",
        (Idioma::Turco, "unmounting") => "Dosya sistemlerinin bağlantısı kesiliyor…",
        (Idioma::Ucraniano, "unmounting") => "Відключення файлових систем…",
        (Idioma::Vietnamita, "unmounting") => "Đang ngắt kết nối hệ thống tập tin…",
        (Idioma::ChinoSimplificado, "unmounting") => "断开文件系统连接...",
        (Idioma::Ingles, "powering_off") => "Turning off the drive…",
        (Idioma::BelarusLatino, "powering_off") => "Vykłjučennje dyska...",
        (Idioma::Belarus, "powering_off") => "Выключэнне дыска...",
        (Idioma::Catalan, "powering_off") => "S'està apagant la unitat...",
        (Idioma::Checo, "powering_off") => "Vypínání disku…",
        (Idioma::Aleman, "powering_off") => "Das Laufwerk wird ausgeschaltet…",
        (Idioma::Frances, "powering_off") => "Mise hors tension du lecteur…",
        (Idioma::Gallego, "powering_off") => "Apagando a unidade...",
        (Idioma::Italiano, "powering_off") => "Spegnimento dell'unità…",
        (Idioma::Coreano, "powering_off") => "드라이브를 끄는 중…",
        (Idioma::Kurdo, "powering_off") => "Vemirandina ajokerê…",
        (Idioma::Neerlandes, "powering_off") => "De schijf uitschakelen…",
        (Idioma::NoruegoNynorsk, "powering_off") => "Slår av stasjonen ...",
        (Idioma::Polaco, "powering_off") => "Wyłączanie napędu…",
        (Idioma::PortuguesBrasil, "powering_off") => "Desligando a unidade…",
        (Idioma::Ruso, "powering_off") => "Выключение накопителя…",
        (Idioma::Sueco, "powering_off") => "Stänger av enheten...",
        (Idioma::Turco, "powering_off") => "Sürücü kapatılıyor…",
        (Idioma::Ucraniano, "powering_off") => "Вимкнення накопичувача…",
        (Idioma::Vietnamita, "powering_off") => "Đang tắt ổ đĩa…",
        (Idioma::ChinoSimplificado, "powering_off") => "关闭驱动器...",
        (Idioma::Ingles, "refreshing_firmware") => "Checking firmware updates…",
        (Idioma::BelarusLatino, "refreshing_firmware") => "Pravjerka abnaŭłjennjaŭ prašyŭki...",
        (Idioma::Belarus, "refreshing_firmware") => "Праверка абнаўленняў прашыўкі…",
        (Idioma::Catalan, "refreshing_firmware") => {
            "S'estan comprovant les actualitzacions del microprogramari..."
        }
        (Idioma::Checo, "refreshing_firmware") => "Kontrola aktualizací firmwaru…",
        (Idioma::Aleman, "refreshing_firmware") => "Firmware-Updates werden überprüft…",
        (Idioma::Frances, "refreshing_firmware") => {
            "Vérification des mises à jour du micrologiciel…"
        }
        (Idioma::Gallego, "refreshing_firmware") => "Comprobando actualizacións de firmware...",
        (Idioma::Italiano, "refreshing_firmware") => "Controllo degli aggiornamenti firmware…",
        (Idioma::Coreano, "refreshing_firmware") => "펌웨어 업데이트 확인 중…",
        (Idioma::Kurdo, "refreshing_firmware") => "Nûvekirinên firmware kontrol dike…",
        (Idioma::Neerlandes, "refreshing_firmware") => "Firmware-updates controleren…",
        (Idioma::NoruegoNynorsk, "refreshing_firmware") => "Sjekker fastvareoppdateringer...",
        (Idioma::Polaco, "refreshing_firmware") => {
            "Sprawdzanie aktualizacji oprogramowania sprzętowego…"
        }
        (Idioma::PortuguesBrasil, "refreshing_firmware") => "Verificando atualizações de firmware…",
        (Idioma::Ruso, "refreshing_firmware") => "Проверка обновлений прошивки…",
        (Idioma::Sueco, "refreshing_firmware") => "Kontrollerar firmwareuppdateringar...",
        (Idioma::Turco, "refreshing_firmware") => "Ürün yazılımı güncellemeleri kontrol ediliyor…",
        (Idioma::Ucraniano, "refreshing_firmware") => "Перевірка оновлень мікропрограми…",
        (Idioma::Vietnamita, "refreshing_firmware") => {
            "Đang kiểm tra bản cập nhật chương trình cơ sở…"
        }
        (Idioma::ChinoSimplificado, "refreshing_firmware") => "检查固件更新...",
        (Idioma::Ingles, "installing_firmware") => "Installing firmware…",
        (Idioma::BelarusLatino, "installing_firmware") => "Ustałjavannje prašyŭki...",
        (Idioma::Belarus, "installing_firmware") => "Усталяванне прашыўкі…",
        (Idioma::Catalan, "installing_firmware") => "S'està instal·lant el microprogramari...",
        (Idioma::Checo, "installing_firmware") => "Instalace firmwaru…",
        (Idioma::Aleman, "installing_firmware") => "Firmware wird installiert…",
        (Idioma::Frances, "installing_firmware") => "Installation du micrologiciel…",
        (Idioma::Gallego, "installing_firmware") => "Instalando firmware...",
        (Idioma::Italiano, "installing_firmware") => "Installazione del firmware…",
        (Idioma::Coreano, "installing_firmware") => "펌웨어 설치 중…",
        (Idioma::Kurdo, "installing_firmware") => "Sazkirina firmware…",
        (Idioma::Neerlandes, "installing_firmware") => "Firmware installeren…",
        (Idioma::NoruegoNynorsk, "installing_firmware") => "Installerer fastvare...",
        (Idioma::Polaco, "installing_firmware") => "Instalowanie oprogramowania sprzętowego…",
        (Idioma::PortuguesBrasil, "installing_firmware") => "Instalando firmware…",
        (Idioma::Ruso, "installing_firmware") => "Установка прошивки…",
        (Idioma::Sueco, "installing_firmware") => "Installerar firmware...",
        (Idioma::Turco, "installing_firmware") => "Aygıt yazılımı yükleniyor…",
        (Idioma::Ucraniano, "installing_firmware") => "Встановлення прошивки…",
        (Idioma::Vietnamita, "installing_firmware") => "Đang cài đặt chương trình cơ sở…",
        (Idioma::ChinoSimplificado, "installing_firmware") => "正在安装固件...",
        (Idioma::Ingles, "scheduling_recovery") => "Preparing recovery for the next restart…",
        (Idioma::BelarusLatino, "scheduling_recovery") => {
            "Padrychtoŭka adnaŭłjennja da nastupnaha pjerazapusku..."
        }
        (Idioma::Belarus, "scheduling_recovery") => {
            "Падрыхтоўка аднаўлення да наступнага перазапуску…"
        }
        (Idioma::Catalan, "scheduling_recovery") => {
            "S'està preparant la recuperació per al proper reinici..."
        }
        (Idioma::Checo, "scheduling_recovery") => "Příprava obnovení na další restart…",
        (Idioma::Aleman, "scheduling_recovery") => {
            "Wiederherstellung wird für den nächsten Neustart vorbereitet…"
        }
        (Idioma::Frances, "scheduling_recovery") => {
            "Préparation de la récupération pour le prochain redémarrage…"
        }
        (Idioma::Gallego, "scheduling_recovery") => {
            "Preparando a recuperación para o próximo reinicio..."
        }
        (Idioma::Italiano, "scheduling_recovery") => {
            "Preparazione del ripristino per il prossimo riavvio…"
        }
        (Idioma::Coreano, "scheduling_recovery") => "다음 재시작을 위한 복구 준비 중…",
        (Idioma::Kurdo, "scheduling_recovery") => {
            "Amadekirina başbûnê ji bo ji nû ve destpêkirina din…"
        }
        (Idioma::Neerlandes, "scheduling_recovery") => {
            "Herstel voorbereiden voor de volgende herstart…"
        }
        (Idioma::NoruegoNynorsk, "scheduling_recovery") => {
            "Forbereder gjenoppretting for neste omstart..."
        }
        (Idioma::Polaco, "scheduling_recovery") => {
            "Przygotowywanie odzyskiwania do następnego ponownego uruchomienia…"
        }
        (Idioma::PortuguesBrasil, "scheduling_recovery") => {
            "Preparando recuperação para a próxima reinicialização…"
        }
        (Idioma::Ruso, "scheduling_recovery") => {
            "Подготовка восстановления к следующему перезапуску…"
        }
        (Idioma::Sueco, "scheduling_recovery") => "Förbereder återställning för nästa omstart...",
        (Idioma::Turco, "scheduling_recovery") => "Preparing recovery for the next restart…",
        (Idioma::Ucraniano, "scheduling_recovery") => {
            "Підготовка відновлення до наступного перезапуску…"
        }
        (Idioma::Vietnamita, "scheduling_recovery") => {
            "Đang chuẩn bị khôi phục cho lần khởi động lại tiếp theo…"
        }
        (Idioma::ChinoSimplificado, "scheduling_recovery") => "正在为下次重新启动准备恢复...",
        (Idioma::Ingles, "testing_sound") => "Testing the selected sound output…",
        (Idioma::BelarusLatino, "testing_sound") => "Testavannje abranaha vychadu huku…",
        (Idioma::Belarus, "testing_sound") => "Тэставанне абранага выхаду гуку…",
        (Idioma::Catalan, "testing_sound") => "S'està provant la sortida de so seleccionada...",
        (Idioma::Checo, "testing_sound") => "Testování vybraného zvukového výstupu…",
        (Idioma::Aleman, "testing_sound") => "Testen der ausgewählten Tonausgabe…",
        (Idioma::Frances, "testing_sound") => "Test de la sortie sonore sélectionnée…",
        (Idioma::Gallego, "testing_sound") => "Probando a saída de son seleccionada...",
        (Idioma::Italiano, "testing_sound") => "Test dell'uscita audio selezionata…",
        (Idioma::Coreano, "testing_sound") => "선택한 사운드 출력을 테스트 중입니다…",
        (Idioma::Kurdo, "testing_sound") => "Ceribandina dengê hilbijartî…",
        (Idioma::Neerlandes, "testing_sound") => "De geselecteerde geluidsuitvoer testen…",
        (Idioma::NoruegoNynorsk, "testing_sound") => "Tester valgt lydutgang...",
        (Idioma::Polaco, "testing_sound") => "Testowanie wybranego wyjścia dźwięku…",
        (Idioma::PortuguesBrasil, "testing_sound") => "Testando a saída de som selecionada…",
        (Idioma::Ruso, "testing_sound") => "Проверка выбранного вывода звука…",
        (Idioma::Sueco, "testing_sound") => "Testar den valda ljudutgången...",
        (Idioma::Turco, "testing_sound") => "Seçilen ses çıkışı test ediliyor…",
        (Idioma::Ucraniano, "testing_sound") => "Перевірка вибраного вихідного звуку…",
        (Idioma::Vietnamita, "testing_sound") => "Kiểm tra đầu ra âm thanh đã chọn…",
        (Idioma::ChinoSimplificado, "testing_sound") => "测试所选的声音输出...",
        (Idioma::Ingles, "recording_mic") => "Recording a temporary microphone sample…",
        (Idioma::BelarusLatino, "recording_mic") => "Idzje zapis časovaha ŭzoru mikrafona…",
        (Idioma::Belarus, "recording_mic") => "Запіс часовага ўзору мікрафона…",
        (Idioma::Catalan, "recording_mic") => "Enregistrament d'una mostra de micròfon temporal...",
        (Idioma::Checo, "recording_mic") => "Nahrávání dočasného vzorku mikrofonu…",
        (Idioma::Aleman, "recording_mic") => "Aufzeichnen einer temporären Mikrofonprobe…",
        (Idioma::Frances, "recording_mic") => {
            "Enregistrement d'un échantillon de microphone temporaire…"
        }
        (Idioma::Gallego, "recording_mic") => "Gravando unha mostra de micrófono temporal...",
        (Idioma::Italiano, "recording_mic") => {
            "Registrazione di un campione temporaneo del microfono…"
        }
        (Idioma::Coreano, "recording_mic") => "임시 마이크 샘플을 녹음하는 중…",
        (Idioma::Kurdo, "recording_mic") => "Tomarkirina nimûneyek mîkrofona demkî…",
        (Idioma::Neerlandes, "recording_mic") => "Een tijdelijk microfoonsample opnemen…",
        (Idioma::NoruegoNynorsk, "recording_mic") => "Tar opp en midlertidig mikrofoneksempel...",
        (Idioma::Polaco, "recording_mic") => "Nagrywam tymczasową próbkę mikrofonu…",
        (Idioma::PortuguesBrasil, "recording_mic") => {
            "Gravando uma amostra temporária de microfone…"
        }
        (Idioma::Ruso, "recording_mic") => "Запись временного сэмпла микрофона…",
        (Idioma::Sueco, "recording_mic") => "Spelar in ett tillfälligt mikrofonprov...",
        (Idioma::Turco, "recording_mic") => "Recording a temporary microphone sample…",
        (Idioma::Ucraniano, "recording_mic") => "Записування тимчасового зразка мікрофона…",
        (Idioma::Vietnamita, "recording_mic") => "Ghi âm mẫu micrô tạm thời…",
        (Idioma::ChinoSimplificado, "recording_mic") => "录制临时麦克风样本...",
        (Idioma::Ingles, "playing_mic") => "Playing the microphone sample…",
        (Idioma::BelarusLatino, "playing_mic") => "Prajhravannje ŭzoru mikrafona…",
        (Idioma::Belarus, "playing_mic") => "Прайграванне ўзору мікрафона…",
        (Idioma::Catalan, "playing_mic") => "S'està reproduint la mostra del micròfon...",
        (Idioma::Checo, "playing_mic") => "Přehrávání ukázky mikrofonu…",
        (Idioma::Aleman, "playing_mic") => "Das Mikrofonbeispiel wird abgespielt…",
        (Idioma::Frances, "playing_mic") => "Lecture de l'échantillon du microphone…",
        (Idioma::Gallego, "playing_mic") => "Reproducindo a mostra do micrófono...",
        (Idioma::Italiano, "playing_mic") => "Riproduzione del campione del microfono…",
        (Idioma::Coreano, "playing_mic") => "마이크 샘플을 재생하는 중…",
        (Idioma::Kurdo, "playing_mic") => "Nimûneya mîkrofonê dileyize…",
        (Idioma::Neerlandes, "playing_mic") => "Het microfoonvoorbeeld afspelen…",
        (Idioma::NoruegoNynorsk, "playing_mic") => "Spiller av mikrofoneksemplet...",
        (Idioma::Polaco, "playing_mic") => "Odtwarzanie próbki mikrofonu…",
        (Idioma::PortuguesBrasil, "playing_mic") => "Reproduzindo a amostra do microfone…",
        (Idioma::Ruso, "playing_mic") => "Воспроизведение семпла с микрофона…",
        (Idioma::Sueco, "playing_mic") => "Spelar upp mikrofonexemplet...",
        (Idioma::Turco, "playing_mic") => "Mikrofon örneği çalınıyor…",
        (Idioma::Ucraniano, "playing_mic") => "Відтворення зразка мікрофона…",
        (Idioma::Vietnamita, "playing_mic") => "Đang phát mẫu micrô…",
        (Idioma::ChinoSimplificado, "playing_mic") => "播放麦克风样本...",
        (Idioma::Ingles, "done") => "Done",
        (Idioma::BelarusLatino, "done") => "Hatova",
        (Idioma::Belarus, "done") => "Гатова",
        (Idioma::Catalan, "done") => "Fet",
        (Idioma::Checo, "done") => "Hotovo",
        (Idioma::Aleman, "done") => "Fertig",
        (Idioma::Frances, "done") => "Fait",
        (Idioma::Gallego, "done") => "Feito",
        (Idioma::Italiano, "done") => "Fatto",
        (Idioma::Coreano, "done") => "완료",
        (Idioma::Kurdo, "done") => "Qediya",
        (Idioma::Neerlandes, "done") => "Gereed",
        (Idioma::NoruegoNynorsk, "done") => "Ferdig",
        (Idioma::Polaco, "done") => "Gotowe",
        (Idioma::PortuguesBrasil, "done") => "Pronto",
        (Idioma::Ruso, "done") => "Готово",
        (Idioma::Sueco, "done") => "Klar",
        (Idioma::Turco, "done") => "Tamamlandı",
        (Idioma::Ucraniano, "done") => "Готово",
        (Idioma::Vietnamita, "done") => "Xong",
        (Idioma::ChinoSimplificado, "done") => "完成",

        (Idioma::Hungaro, "reading") => "A számítógép adatainak olvasása…",
        (Idioma::Hungaro, "preparing") => "Előkészítés…",
        (Idioma::Hungaro, "changing_channel") => "A rendszercsatorna módosítása…",
        (Idioma::Hungaro, "updating_catalog") => "A szoftverkatalógus frissítése…",
        (Idioma::Hungaro, "validating") => "A frissített beállítás ellenőrzése…",
        (Idioma::Hungaro, "cleaning_versions") => "Régi rendszerverziók eltávolítása…",
        (Idioma::Hungaro, "garbage_collect") => "A nem használt hely felszabadítása…",
        (Idioma::Hungaro, "optimising_store") => "A rendszertároló optimalizálása…",
        (Idioma::Hungaro, "saving_data") => "A függőben lévő írások befejezése…",
        (Idioma::Hungaro, "unmounting") => "Fájlrendszerek leválasztása…",
        (Idioma::Hungaro, "powering_off") => "A meghajtó kikapcsolása…",
        (Idioma::Hungaro, "refreshing_firmware") => "Firmware-frissítések ellenőrzése…",
        (Idioma::Hungaro, "installing_firmware") => "Firmware telepítése…",
        (Idioma::Hungaro, "scheduling_recovery") => {
            "Helyreállítás előkészítése a következő újraindításhoz…"
        }
        (Idioma::Hungaro, "testing_sound") => "A kiválasztott hangkimenet tesztelése…",
        (Idioma::Hungaro, "recording_mic") => "Ideiglenes mikrofonminta felvétele…",
        (Idioma::Hungaro, "playing_mic") => "A mikrofonminta lejátszása…",
        (Idioma::Hungaro, "done") => "Kész",

        (_, "reading") => "Leyendo este equipo…",
        (_, "preparing") => "Preparando…",
        (_, "changing_channel") => "Cambiando el canal del sistema…",
        (_, "updating_catalog") => "Actualizando el catálogo de software…",
        (_, "validating") => "Comprobando la configuración actualizada…",
        (_, "cleaning_versions") => "Eliminando versiones antiguas del sistema…",
        (_, "garbage_collect") => "Liberando espacio que ya no se usa…",
        (_, "optimising_store") => "Optimizando el almacenamiento del sistema…",
        (_, "saving_data") => "Terminando de guardar los datos pendientes…",
        (_, "unmounting") => "Desconectando los sistemas de archivos…",
        (_, "powering_off") => "Apagando la unidad…",
        (_, "refreshing_firmware") => "Comprobando actualizaciones de firmware…",
        (_, "installing_firmware") => "Instalando firmware…",
        (_, "scheduling_recovery") => "Preparando la recuperación para el próximo reinicio…",
        (_, "testing_sound") => "Probando la salida de sonido seleccionada…",
        (_, "recording_mic") => "Grabando una prueba temporal del micrófono…",
        (_, "playing_mic") => "Reproduciendo la prueba del micrófono…",
        (_, "done") => "Listo",
        _ => "Korunix",
    }
}

fn mostrar_progreso(estado: &Estado, porcentaje: u8, clave: &str) {
    let porcentaje = porcentaje.min(100);

    estado
        .progreso_texto
        .set_text(texto_progreso(estado.idioma, clave));
    estado
        .progreso_barra
        .set_fraction(f64::from(porcentaje) / 100.0);
    estado
        .progreso_barra
        .set_text(Some(&format!("{porcentaje}%")));
    estado.progreso.set_reveal_child(true);
}

fn ocultar_progreso(estado: &Estado) {
    estado.progreso.set_reveal_child(false);
    estado.progreso_barra.set_fraction(0.0);
    estado.progreso_barra.set_text(Some("0%"));
}

fn etapa_inicial_motor(argumentos: &[&str]) -> Option<&'static str> {
    match argumentos {
        ["channel", _, "--yes"] => Some("changing_channel"),
        ["update", "--json"] => Some("updating_catalog"),
        ["clean", "--yes", "--json"] | ["clean-all", "--yes", "--json"] => {
            Some("cleaning_versions")
        }
        ["storage", "eject", _, "--yes", "--json"] => Some("unmounting"),
        ["storage", "eject", _, "--heavy", "--yes", "--json"] => Some("saving_data"),
        ["firmware", "refresh", "--yes", "--json"] => Some("refreshing_firmware"),
        ["firmware", "update", _, "--yes", "--json"] => Some("installing_firmware"),
        ["media", "audio", "test-output", ..] => Some("testing_sound"),
        ["rollback", _, "--yes", "--json"] => Some("scheduling_recovery"),
        _ => None,
    }
}

fn ejecutar_motor_trabajador(
    motor: PathBuf,
    raiz: PathBuf,
    argumentos: Vec<String>,
    emisor: mpsc::Sender<EventoMotor>,
) {
    let resultado = (|| -> Result<String, String> {
        let mut hijo = Command::new(&motor)
            .args(&argumentos)
            .current_dir(&raiz)
            .env("KORUNIX_ROOT", &raiz)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("No pude iniciar el motor: {error}"))?;

        let stdout = hijo
            .stdout
            .take()
            .ok_or_else(|| "No pude leer la salida del motor.".to_string())?;
        let stderr = hijo
            .stderr
            .take()
            .ok_or_else(|| "No pude leer los detalles del motor.".to_string())?;

        let mut detalles = Vec::<String>::new();

        for linea in BufReader::new(stderr).lines() {
            let linea = linea.map_err(|error| format!("No pude leer el motor: {error}"))?;

            if let Some(resto) = linea.strip_prefix("KORUNIX_PROGRESS\t") {
                let mut partes = resto.splitn(2, '\t');
                let porcentaje = partes.next().and_then(|valor| valor.parse::<u8>().ok());
                let etapa = partes.next();

                if let (Some(porcentaje), Some(etapa)) = (porcentaje, etapa) {
                    let _ = emisor.send(EventoMotor::Progreso(porcentaje, etapa.to_string()));
                    continue;
                }
            }

            if !linea.trim().is_empty() {
                detalles.push(linea);
                if detalles.len() > 80 {
                    detalles.remove(0);
                }
            }
        }

        let mut salida = String::new();
        BufReader::new(stdout)
            .read_to_string(&mut salida)
            .map_err(|error| format!("No pude leer la salida del motor: {error}"))?;

        let estado = hijo
            .wait()
            .map_err(|error| format!("No pude esperar al motor: {error}"))?;

        if !estado.success() {
            return Err(if detalles.is_empty() {
                "El motor terminó con error.".to_string()
            } else {
                detalles.join("\n")
            });
        }

        Ok(salida.trim().to_string())
    })();

    let _ = emisor.send(EventoMotor::Terminado(resultado));
}

fn ejecutar_motor(estado: &Estado, argumentos: &[&str]) -> Result<String, String> {
    if estado.ocupado.replace(true) {
        return Err("Korunix ya está realizando otra operación.".to_string());
    }

    let etapa_inicial = etapa_inicial_motor(argumentos);

    if let Some(etapa) = etapa_inicial {
        mostrar_progreso(estado, 5, etapa);
    }

    let es_multimedia = argumentos.first().copied() == Some("media");
    let sensible_anterior = estado.stack.is_sensitive();
    if !es_multimedia {
        estado.stack.set_sensitive(false);
    }

    let motor = estado.motor.clone();
    let raiz = estado.raiz.clone();
    let argumentos_hilo = argumentos
        .iter()
        .map(|argumento| (*argumento).to_string())
        .collect::<Vec<_>>();

    let (emisor, receptor) = mpsc::channel::<EventoMotor>();

    thread::spawn(move || {
        ejecutar_motor_trabajador(motor, raiz, argumentos_hilo, emisor);
    });

    let contexto = glib::MainContext::default();
    let resultado = loop {
        match receptor.try_recv() {
            Ok(EventoMotor::Progreso(porcentaje, etapa)) => {
                mostrar_progreso(estado, porcentaje, &etapa);
            }
            Ok(EventoMotor::Terminado(resultado)) => break resultado,
            Err(TryRecvError::Empty) => {
                while contexto.pending() {
                    contexto.iteration(false);
                }
                thread::sleep(Duration::from_millis(12));
            }
            Err(TryRecvError::Disconnected) => {
                break Err("El proceso de Korunix terminó inesperadamente.".to_string());
            }
        }
    };

    if etapa_inicial.is_some() && resultado.is_ok() {
        mostrar_progreso(estado, 100, "done");
    }

    if etapa_inicial.is_some() {
        ocultar_progreso(estado);
    }

    if !es_multimedia {
        estado.stack.set_sensitive(sensible_anterior);
    }
    estado.ocupado.set(false);

    resultado
}

fn consultar(estado: &Estado, area: &str) -> Result<Value, String> {
    let salida = ejecutar_motor(estado, &[area, "--json"])?;

    serde_json::from_str(&salida)
        .map_err(|error| format!("El motor devolvió JSON inválido para {area}: {error}"))
}

fn ejecutar_json(estado: &Estado, argumentos: &[&str]) -> Result<Value, String> {
    let salida = ejecutar_motor(estado, argumentos)?;

    serde_json::from_str(&salida)
        .map_err(|error| format!("El motor devolvió JSON inválido: {error}"))
}

fn cantidad(datos: &Value, puntero: &str) -> usize {
    datos
        .pointer(puntero)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn modelo_cadenas(valores: &[String]) -> gtk::StringList {
    let referencias = valores.iter().map(String::as_str).collect::<Vec<_>>();
    gtk::StringList::new(&referencias)
}

fn dialogo_confirmacion(
    boton: &gtk::Button,
    idioma: Idioma,
    cuerpo: &str,
    accion: &str,
    destructiva: bool,
) -> adw::MessageDialog {
    let ventana = boton
        .root()
        .and_then(|raiz| raiz.downcast::<gtk::Window>().ok());
    let cuerpo = localizar_visible(idioma, cuerpo);
    let accion = localizar_visible(idioma, accion);
    let dialogo = adw::MessageDialog::new(
        ventana.as_ref(),
        Some(texto(idioma, "confirm_operation")),
        Some(cuerpo.as_str()),
    );

    dialogo.set_close_response("cancel");
    dialogo.set_default_response(Some("apply"));
    dialogo.add_responses(&[
        ("cancel", texto(idioma, "cancel")),
        ("apply", accion.as_str()),
    ]);
    dialogo.set_response_appearance(
        "apply",
        if destructiva {
            adw::ResponseAppearance::Destructive
        } else {
            adw::ResponseAppearance::Suggested
        },
    );
    dialogo
}

fn mostrar_error(estado: &Estado, error: impl AsRef<str>) {
    let detalle = error.as_ref().to_string();
    let ventana = estado
        .stack
        .root()
        .and_then(|raiz| raiz.downcast::<gtk::Window>().ok());

    let cerrar = match estado.idioma {
        Idioma::Ingles => "Close",
        Idioma::BelarusLatino => "Błizka",
        Idioma::Belarus => "Закрыць",
        Idioma::Catalan => "Tanca",
        Idioma::Checo => "Zavřít",
        Idioma::Aleman => "Schließen",
        Idioma::Frances => "Fermer",
        Idioma::Gallego => "Pechar",
        Idioma::Italiano => "Chiudi",
        Idioma::Coreano => "닫기",
        Idioma::Kurdo => "Bigire",
        Idioma::Neerlandes => "Sluiten",
        Idioma::NoruegoNynorsk => "Lat att",
        Idioma::Polaco => "Zamknij",
        Idioma::PortuguesBrasil => "Fechar",
        Idioma::Ruso => "Закрыть",
        Idioma::Sueco => "Stäng",
        Idioma::Turco => "Kapat",
        Idioma::Ucraniano => "Закрити",
        Idioma::Vietnamita => "Đóng",
        Idioma::ChinoSimplificado => "关闭",
        Idioma::Hungaro => "Bezárás",
        Idioma::Espanol => "Cerrar",
    };

    let dialogo = adw::MessageDialog::new(
        ventana.as_ref(),
        Some(texto(estado.idioma, "error")),
        Some(texto_error_amigable(estado.idioma)),
    );

    dialogo.set_close_response("close");
    dialogo.set_default_response(Some("close"));
    dialogo.add_responses(&[
        ("close", cerrar),
        ("details", texto_detalles_tecnicos(estado.idioma)),
    ]);

    let idioma = estado.idioma;
    let padre = ventana.clone();

    dialogo.connect_response(Some("details"), move |_, _| {
        let detalle_dialogo = adw::MessageDialog::new(
            padre.as_ref(),
            Some(texto_detalles_tecnicos(idioma)),
            Some(&detalle),
        );

        let cerrar = match idioma {
            Idioma::Ingles => "Close",
            Idioma::BelarusLatino => "Błizka",
            Idioma::Belarus => "Закрыць",
            Idioma::Catalan => "Tanca",
            Idioma::Checo => "Zavřít",
            Idioma::Aleman => "Schließen",
            Idioma::Frances => "Fermer",
            Idioma::Gallego => "Pechar",
            Idioma::Italiano => "Chiudi",
            Idioma::Coreano => "닫기",
            Idioma::Kurdo => "Bigire",
            Idioma::Neerlandes => "Sluiten",
            Idioma::NoruegoNynorsk => "Lat att",
            Idioma::Polaco => "Zamknij",
            Idioma::PortuguesBrasil => "Fechar",
            Idioma::Ruso => "Закрыть",
            Idioma::Sueco => "Stäng",
            Idioma::Turco => "Kapat",
            Idioma::Ucraniano => "Закрити",
            Idioma::Vietnamita => "Đóng",
            Idioma::ChinoSimplificado => "关闭",
            Idioma::Hungaro => "Bezárás",
            Idioma::Espanol => "Cerrar",
        };

        detalle_dialogo.set_close_response("close");
        detalle_dialogo.set_default_response(Some("close"));
        detalle_dialogo.add_response("close", cerrar);
        detalle_dialogo.present();
    });

    dialogo.present();
}

fn mostrar_exito(estado: &Estado, mensaje: &str) {
    let mensaje = localizar_visible(estado.idioma, mensaje);
    estado.toast.add_toast(adw::Toast::new(&mensaje));
}

fn valor(datos: &Value, puntero: &str) -> String {
    let Some(valor) = datos.pointer(puntero) else {
        return "—".to_string();
    };

    match valor {
        Value::Null => "—".to_string(),
        Value::String(texto) if texto.is_empty() => "—".to_string(),
        Value::String(texto) => texto.clone(),
        Value::Bool(valor) => valor.to_string(),
        Value::Number(valor) => valor.to_string(),
        Value::Array(valores) => valores
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", "),
        otro => otro.to_string(),
    }
}

fn memoria_humana(datos: &Value) -> String {
    let bytes = datos
        .pointer("/memory/bytes")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    if bytes == 0 {
        "—".to_string()
    } else {
        format!("{:.1} GiB", bytes as f64 / 1024_f64.powi(3),)
    }
}

fn fabricante_humano(valor: &str) -> String {
    match valor.trim() {
        "Gigabyte Technology Co., Ltd." => "Gigabyte".to_string(),
        "Micro-Star International Co., Ltd." => "MSI".to_string(),
        "ASUSTeK COMPUTER INC." => "ASUS".to_string(),
        "LENOVO" => "Lenovo".to_string(),
        otro => otro.to_string(),
    }
}

fn modelo_humano(vendor: &str, model: &str) -> String {
    let fabricante = fabricante_humano(vendor);
    let mut partes = Vec::new();

    for parte in [fabricante.as_str(), model.trim()] {
        if !parte.is_empty() && parte != "—" && !partes.iter().any(|actual| actual == &parte) {
            partes.push(parte);
        }
    }

    if partes.is_empty() {
        "—".to_string()
    } else {
        partes.join(" ")
    }
}

fn idioma_humano(idioma: Idioma, valor: &str) -> String {
    let codigo = valor
        .split(['_', '-', '.'])
        .next()
        .unwrap_or(valor)
        .to_ascii_lowercase();

    match (idioma, codigo.as_str()) {
        (Idioma::Ingles, "es") => "Spanish".to_string(),
        (Idioma::BelarusLatino, "es") => "ispanski".to_string(),
        (Idioma::Belarus, "es") => "Іспанская".to_string(),
        (Idioma::Catalan, "es") => "espanyol".to_string(),
        (Idioma::Checo, "es") => "Španělština".to_string(),
        (Idioma::Aleman, "es") => "Spanisch".to_string(),
        (Idioma::Frances, "es") => "espagnol".to_string(),
        (Idioma::Gallego, "es") => "castelán".to_string(),
        (Idioma::Italiano, "es") => "Spagnolo".to_string(),
        (Idioma::Coreano, "es") => "스페인어".to_string(),
        (Idioma::Kurdo, "es") => "Spanî".to_string(),
        (Idioma::Neerlandes, "es") => "Spaans".to_string(),
        (Idioma::NoruegoNynorsk, "es") => "Spansk".to_string(),
        (Idioma::Polaco, "es") => "Hiszpański".to_string(),
        (Idioma::PortuguesBrasil, "es") => "Espanhol".to_string(),
        (Idioma::Ruso, "es") => "Испанский".to_string(),
        (Idioma::Sueco, "es") => "Spanska".to_string(),
        (Idioma::Turco, "es") => "İspanyolca".to_string(),
        (Idioma::Ucraniano, "es") => "Іспанська".to_string(),
        (Idioma::Vietnamita, "es") => "Tây Ban Nha".to_string(),
        (Idioma::ChinoSimplificado, "es") => "西班牙语".to_string(),
        (Idioma::Ingles, "en") => "English".to_string(),
        (Idioma::BelarusLatino, "en") => "anhłijskaja".to_string(),
        (Idioma::Belarus, "en") => "англійская".to_string(),
        (Idioma::Catalan, "en") => "Anglès".to_string(),
        (Idioma::Checo, "en") => "anglicky".to_string(),
        (Idioma::Aleman, "en") => "Englisch".to_string(),
        (Idioma::Frances, "en") => "Anglais".to_string(),
        (Idioma::Gallego, "en") => "inglés".to_string(),
        (Idioma::Italiano, "en") => "Inglese".to_string(),
        (Idioma::Coreano, "en") => "영어".to_string(),
        (Idioma::Kurdo, "en") => "Îngilîzî".to_string(),
        (Idioma::Neerlandes, "en") => "Engels".to_string(),
        (Idioma::NoruegoNynorsk, "en") => "Engelsk".to_string(),
        (Idioma::Polaco, "en") => "Angielski".to_string(),
        (Idioma::PortuguesBrasil, "en") => "Inglês".to_string(),
        (Idioma::Ruso, "en") => "английский".to_string(),
        (Idioma::Sueco, "en") => "Engelska".to_string(),
        (Idioma::Turco, "en") => "İngilizce".to_string(),
        (Idioma::Ucraniano, "en") => "англійська".to_string(),
        (Idioma::Vietnamita, "en") => "Tiếng Anh".to_string(),
        (Idioma::ChinoSimplificado, "en") => "英语".to_string(),
        (Idioma::Ingles, "hu") => "Hungarian".to_string(),
        (Idioma::BelarusLatino, "hu") => "vjenhjerski".to_string(),
        (Idioma::Belarus, "hu") => "Венгерская".to_string(),
        (Idioma::Catalan, "hu") => "hongarès".to_string(),
        (Idioma::Checo, "hu") => "maďarština".to_string(),
        (Idioma::Aleman, "hu") => "Ungarisch".to_string(),
        (Idioma::Frances, "hu") => "hongrois".to_string(),
        (Idioma::Gallego, "hu") => "húngaro".to_string(),
        (Idioma::Italiano, "hu") => "Ungherese".to_string(),
        (Idioma::Coreano, "hu") => "헝가리어".to_string(),
        (Idioma::Kurdo, "hu") => "Macarî".to_string(),
        (Idioma::Neerlandes, "hu") => "Hongaars".to_string(),
        (Idioma::NoruegoNynorsk, "hu") => "Ungarsk".to_string(),
        (Idioma::Polaco, "hu") => "węgierski".to_string(),
        (Idioma::PortuguesBrasil, "hu") => "Húngaro".to_string(),
        (Idioma::Ruso, "hu") => "Венгерский".to_string(),
        (Idioma::Sueco, "hu") => "Ungerska".to_string(),
        (Idioma::Turco, "hu") => "Macarca".to_string(),
        (Idioma::Ucraniano, "hu") => "Угорська".to_string(),
        (Idioma::Vietnamita, "hu") => "Hungary".to_string(),
        (Idioma::ChinoSimplificado, "hu") => "匈牙利语".to_string(),
        (Idioma::Hungaro, "es") => "Spanyol".to_string(),
        (Idioma::Hungaro, "en") => "Angol".to_string(),
        (Idioma::Hungaro, "hu") => "Magyar".to_string(),
        (_, "es") => "Español".to_string(),
        (_, "en") => "Inglés".to_string(),
        (_, "hu") => "Húngaro".to_string(),
        _ => valor.to_string(),
    }
}

fn region_humana(idioma: Idioma, valor: &str) -> String {
    let codigo = valor.trim().to_ascii_uppercase();

    match (idioma, codigo.as_str()) {
        (Idioma::Ingles, "PE") => "Peru".to_string(),
        (Idioma::BelarusLatino, "PE") => "Pjeru".to_string(),
        (Idioma::Belarus, "PE") => "Перу".to_string(),
        (Idioma::Catalan, "PE") => "Perú".to_string(),
        (Idioma::Checo, "PE") => "Peru".to_string(),
        (Idioma::Aleman, "PE") => "Peru".to_string(),
        (Idioma::Frances, "PE") => "Pérou".to_string(),
        (Idioma::Gallego, "PE") => "Perú".to_string(),
        (Idioma::Italiano, "PE") => "Perù".to_string(),
        (Idioma::Coreano, "PE") => "페루".to_string(),
        (Idioma::Kurdo, "PE") => "Perû".to_string(),
        (Idioma::Neerlandes, "PE") => "Peru".to_string(),
        (Idioma::NoruegoNynorsk, "PE") => "Peru".to_string(),
        (Idioma::Polaco, "PE") => "Peru".to_string(),
        (Idioma::PortuguesBrasil, "PE") => "Peru".to_string(),
        (Idioma::Ruso, "PE") => "Перу".to_string(),
        (Idioma::Sueco, "PE") => "Peru".to_string(),
        (Idioma::Turco, "PE") => "Peru".to_string(),
        (Idioma::Ucraniano, "PE") => "Перу".to_string(),
        (Idioma::Vietnamita, "PE") => "Peru".to_string(),
        (Idioma::ChinoSimplificado, "PE") => "秘鲁".to_string(),
        (Idioma::Ingles, "ES") => "Spain".to_string(),
        (Idioma::BelarusLatino, "ES") => "Ispanija".to_string(),
        (Idioma::Belarus, "ES") => "Іспанія".to_string(),
        (Idioma::Catalan, "ES") => "Espanya".to_string(),
        (Idioma::Checo, "ES") => "Španělsko".to_string(),
        (Idioma::Aleman, "ES") => "Spanien".to_string(),
        (Idioma::Frances, "ES") => "Espagne".to_string(),
        (Idioma::Gallego, "ES") => "España".to_string(),
        (Idioma::Italiano, "ES") => "Spagna".to_string(),
        (Idioma::Coreano, "ES") => "스페인".to_string(),
        (Idioma::Kurdo, "ES") => "Spanya".to_string(),
        (Idioma::Neerlandes, "ES") => "Spanje".to_string(),
        (Idioma::NoruegoNynorsk, "ES") => "Spania".to_string(),
        (Idioma::Polaco, "ES") => "Hiszpania".to_string(),
        (Idioma::PortuguesBrasil, "ES") => "Espanha".to_string(),
        (Idioma::Ruso, "ES") => "Испания".to_string(),
        (Idioma::Sueco, "ES") => "Spanien".to_string(),
        (Idioma::Turco, "ES") => "İspanya".to_string(),
        (Idioma::Ucraniano, "ES") => "Іспанія".to_string(),
        (Idioma::Vietnamita, "ES") => "Tây Ban Nha".to_string(),
        (Idioma::ChinoSimplificado, "ES") => "西班牙".to_string(),
        (Idioma::Ingles, "US") => "United States".to_string(),
        (Idioma::BelarusLatino, "US") => "ZŠA".to_string(),
        (Idioma::Belarus, "US") => "Злучаныя Штаты".to_string(),
        (Idioma::Catalan, "US") => "Estats Units".to_string(),
        (Idioma::Checo, "US") => "Spojené státy americké".to_string(),
        (Idioma::Aleman, "US") => "Vereinigte Staaten".to_string(),
        (Idioma::Frances, "US") => "États-Unis".to_string(),
        (Idioma::Gallego, "US") => "Estados Unidos".to_string(),
        (Idioma::Italiano, "US") => "Stati Uniti".to_string(),
        (Idioma::Coreano, "US") => "미국".to_string(),
        (Idioma::Kurdo, "US") => "Dewletên Yekbûyî".to_string(),
        (Idioma::Neerlandes, "US") => "Verenigde Staten".to_string(),
        (Idioma::NoruegoNynorsk, "US") => "USA".to_string(),
        (Idioma::Polaco, "US") => "Stany Zjednoczone".to_string(),
        (Idioma::PortuguesBrasil, "US") => "Estados Unidos".to_string(),
        (Idioma::Ruso, "US") => "США".to_string(),
        (Idioma::Sueco, "US") => "USA".to_string(),
        (Idioma::Turco, "US") => "Amerika Birleşik Devletleri".to_string(),
        (Idioma::Ucraniano, "US") => "США".to_string(),
        (Idioma::Vietnamita, "US") => "Hoa Kỳ".to_string(),
        (Idioma::ChinoSimplificado, "US") => "美国".to_string(),
        (Idioma::Ingles, "HU") => "Hungary".to_string(),
        (Idioma::BelarusLatino, "HU") => "Vjenhryja".to_string(),
        (Idioma::Belarus, "HU") => "Венгрыя".to_string(),
        (Idioma::Catalan, "HU") => "Hongria".to_string(),
        (Idioma::Checo, "HU") => "Maďarsko".to_string(),
        (Idioma::Aleman, "HU") => "Ungarn".to_string(),
        (Idioma::Frances, "HU") => "Hongrie".to_string(),
        (Idioma::Gallego, "HU") => "Hungría".to_string(),
        (Idioma::Italiano, "HU") => "Ungheria".to_string(),
        (Idioma::Coreano, "HU") => "헝가리".to_string(),
        (Idioma::Kurdo, "HU") => "Macarîstan".to_string(),
        (Idioma::Neerlandes, "HU") => "Hongarije".to_string(),
        (Idioma::NoruegoNynorsk, "HU") => "Ungarn".to_string(),
        (Idioma::Polaco, "HU") => "Węgry".to_string(),
        (Idioma::PortuguesBrasil, "HU") => "Hungria".to_string(),
        (Idioma::Ruso, "HU") => "Венгрия".to_string(),
        (Idioma::Sueco, "HU") => "Ungern".to_string(),
        (Idioma::Turco, "HU") => "Macaristan".to_string(),
        (Idioma::Ucraniano, "HU") => "Угорщина".to_string(),
        (Idioma::Vietnamita, "HU") => "Hungary".to_string(),
        (Idioma::ChinoSimplificado, "HU") => "匈牙利".to_string(),
        (Idioma::Hungaro, "PE") => "Peru".to_string(),
        (Idioma::Hungaro, "ES") => "Spanyolország".to_string(),
        (Idioma::Hungaro, "US") => "Egyesült Államok".to_string(),
        (Idioma::Hungaro, "HU") => "Magyarország".to_string(),
        (_, "PE") => "Perú".to_string(),
        (_, "ES") => "España".to_string(),
        (_, "US") => "Estados Unidos".to_string(),
        (_, "HU") => "Hungría".to_string(),
        _ => valor.to_string(),
    }
}

fn zona_horaria_humana(idioma: Idioma, valor: &str) -> String {
    match (idioma, valor) {
        (Idioma::Ingles, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::BelarusLatino, "America/Lima") => "Łima, Pjeru".to_string(),
        (Idioma::Belarus, "America/Lima") => "Ліма, Перу".to_string(),
        (Idioma::Catalan, "America/Lima") => "Lima, Perú".to_string(),
        (Idioma::Checo, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Aleman, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Frances, "America/Lima") => "Lima, Pérou".to_string(),
        (Idioma::Gallego, "America/Lima") => "Lima, Perú".to_string(),
        (Idioma::Italiano, "America/Lima") => "Lima, Perù".to_string(),
        (Idioma::Coreano, "America/Lima") => "페루 리마".to_string(),
        (Idioma::Kurdo, "America/Lima") => "Lima, Perû".to_string(),
        (Idioma::Neerlandes, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::NoruegoNynorsk, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Polaco, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::PortuguesBrasil, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Ruso, "America/Lima") => "Лима, Перу".to_string(),
        (Idioma::Sueco, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Turco, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Ucraniano, "America/Lima") => "Ліма, Перу".to_string(),
        (Idioma::Vietnamita, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::ChinoSimplificado, "America/Lima") => "秘鲁利马".to_string(),
        (Idioma::Ingles, "Europe/Madrid") => "Madrid, Spain".to_string(),
        (Idioma::BelarusLatino, "Europe/Madrid") => "Madryd, Ispanija".to_string(),
        (Idioma::Belarus, "Europe/Madrid") => "Мадрыд, Іспанія".to_string(),
        (Idioma::Catalan, "Europe/Madrid") => "Madrid, Espanya".to_string(),
        (Idioma::Checo, "Europe/Madrid") => "Madrid, Španělsko".to_string(),
        (Idioma::Aleman, "Europe/Madrid") => "Madrid, Spanien".to_string(),
        (Idioma::Frances, "Europe/Madrid") => "Madrid, Espagne".to_string(),
        (Idioma::Gallego, "Europe/Madrid") => "Madrid, España".to_string(),
        (Idioma::Italiano, "Europe/Madrid") => "Madrid, Spagna".to_string(),
        (Idioma::Coreano, "Europe/Madrid") => "마드리드, 스페인".to_string(),
        (Idioma::Kurdo, "Europe/Madrid") => "Madrid, Spanya".to_string(),
        (Idioma::Neerlandes, "Europe/Madrid") => "Madrid, Spanje".to_string(),
        (Idioma::NoruegoNynorsk, "Europe/Madrid") => "Madrid, Spania".to_string(),
        (Idioma::Polaco, "Europe/Madrid") => "Madryt, Hiszpania".to_string(),
        (Idioma::PortuguesBrasil, "Europe/Madrid") => "Madri, Espanha".to_string(),
        (Idioma::Ruso, "Europe/Madrid") => "Мадрид, Испания".to_string(),
        (Idioma::Sueco, "Europe/Madrid") => "Madrid, Spanien".to_string(),
        (Idioma::Turco, "Europe/Madrid") => "Madrid, İspanya".to_string(),
        (Idioma::Ucraniano, "Europe/Madrid") => "Мадрид, Іспанія".to_string(),
        (Idioma::Vietnamita, "Europe/Madrid") => "Madrid, Tây Ban Nha".to_string(),
        (Idioma::ChinoSimplificado, "Europe/Madrid") => "西班牙马德里".to_string(),
        (Idioma::Ingles, "Europe/Budapest") => "Budapest, Hungary".to_string(),
        (Idioma::BelarusLatino, "Europe/Budapest") => "Budapješt, Vjenhryja".to_string(),
        (Idioma::Belarus, "Europe/Budapest") => "Будапешт, Венгрыя".to_string(),
        (Idioma::Catalan, "Europe/Budapest") => "Budapest, Hongria".to_string(),
        (Idioma::Checo, "Europe/Budapest") => "Budapešť, Maďarsko".to_string(),
        (Idioma::Aleman, "Europe/Budapest") => "Budapest, Ungarn".to_string(),
        (Idioma::Frances, "Europe/Budapest") => "Budapest, Hongrie".to_string(),
        (Idioma::Gallego, "Europe/Budapest") => "Budapest, Hungría".to_string(),
        (Idioma::Italiano, "Europe/Budapest") => "Budapest, Ungheria".to_string(),
        (Idioma::Coreano, "Europe/Budapest") => "부다페스트, 헝가리".to_string(),
        (Idioma::Kurdo, "Europe/Budapest") => "Budapest, Macaristan".to_string(),
        (Idioma::Neerlandes, "Europe/Budapest") => "Boedapest, Hongarije".to_string(),
        (Idioma::NoruegoNynorsk, "Europe/Budapest") => "Budapest, Ungarn".to_string(),
        (Idioma::Polaco, "Europe/Budapest") => "Budapeszt, Węgry".to_string(),
        (Idioma::PortuguesBrasil, "Europe/Budapest") => "Budapeste, Hungria".to_string(),
        (Idioma::Ruso, "Europe/Budapest") => "Будапешт, Венгрия".to_string(),
        (Idioma::Sueco, "Europe/Budapest") => "Budapest, Ungern".to_string(),
        (Idioma::Turco, "Europe/Budapest") => "Budapest, Hungary".to_string(),
        (Idioma::Ucraniano, "Europe/Budapest") => "Будапешт, Угорщина".to_string(),
        (Idioma::Vietnamita, "Europe/Budapest") => "Budapest, Hungary".to_string(),
        (Idioma::ChinoSimplificado, "Europe/Budapest") => "匈牙利布达佩斯".to_string(),
        (Idioma::Hungaro, "America/Lima") => "Lima, Peru".to_string(),
        (Idioma::Hungaro, "Europe/Madrid") => "Madrid, Spanyolország".to_string(),
        (Idioma::Hungaro, "Europe/Budapest") => "Budapest, Magyarország".to_string(),
        (_, "America/Lima") => "Lima, Perú".to_string(),
        (_, "Europe/Madrid") => "Madrid, España".to_string(),
        (_, "Europe/Budapest") => "Budapest, Hungría".to_string(),
        _ => valor.rsplit('/').next().unwrap_or(valor).replace('_', " "),
    }
}

fn teclado_humano(idioma: Idioma, valor: &str) -> String {
    let codigo = valor.to_ascii_lowercase();

    if codigo.contains("latam") {
        return match idioma {
            Idioma::Ingles => "Spanish (Latin America)".to_string(),
            Idioma::BelarusLatino => "Ispanskaja (Łacinskaja Amjeryka)".to_string(),
            Idioma::Belarus => "Іспанская (Лацінская Амерыка)".to_string(),
            Idioma::Catalan => "espanyol (Amèrica Llatina)".to_string(),
            Idioma::Checo => "španělština (Latinská Amerika)".to_string(),
            Idioma::Aleman => "Spanisch (Lateinamerika)".to_string(),
            Idioma::Frances => "Espagnol (Amérique latine)".to_string(),
            Idioma::Gallego => "Español (Latinoamérica)".to_string(),
            Idioma::Italiano => "Spagnolo (America Latina)".to_string(),
            Idioma::Coreano => "스페인어(라틴 아메리카)".to_string(),
            Idioma::Kurdo => "Spanî (Amerîkaya Latîn)".to_string(),
            Idioma::Neerlandes => "Spaans (Latijns-Amerika)".to_string(),
            Idioma::NoruegoNynorsk => "Spansk (Latin-Amerika)".to_string(),
            Idioma::Polaco => "Hiszpański (Ameryka Łacińska)".to_string(),
            Idioma::PortuguesBrasil => "Espanhol (América Latina)".to_string(),
            Idioma::Ruso => "Испанский (Латинская Америка)".to_string(),
            Idioma::Sueco => "Spanska (Latinamerika)".to_string(),
            Idioma::Turco => "İspanyolca (Latin Amerika)".to_string(),
            Idioma::Ucraniano => "Іспанська (Латинська Америка)".to_string(),
            Idioma::Vietnamita => "Tiếng Tây Ban Nha (Mỹ Latinh)".to_string(),
            Idioma::ChinoSimplificado => "西班牙语（拉丁美洲）".to_string(),
            Idioma::Hungaro => "Spanyol (Latin-Amerika)".to_string(),
            Idioma::Espanol => "Español (Latinoamérica)".to_string(),
        };
    }

    if codigo == "es" || codigo.starts_with("es,") {
        return match idioma {
            Idioma::Ingles => "Spanish (Spain)".to_string(),
            Idioma::BelarusLatino => "Ispanskaja (Ispanija)".to_string(),
            Idioma::Belarus => "Іспанская (Іспанія)".to_string(),
            Idioma::Catalan => "Espanyol (Espanya)".to_string(),
            Idioma::Checo => "španělština (Španělsko)".to_string(),
            Idioma::Aleman => "Spanisch (Spanien)".to_string(),
            Idioma::Frances => "Espagnol (Espagne)".to_string(),
            Idioma::Gallego => "Español (España)".to_string(),
            Idioma::Italiano => "Spagnolo (Spagna)".to_string(),
            Idioma::Coreano => "스페인어(스페인)".to_string(),
            Idioma::Kurdo => "Spanî (Ispanya)".to_string(),
            Idioma::Neerlandes => "Spaans (Spanje)".to_string(),
            Idioma::NoruegoNynorsk => "Spansk (Spania)".to_string(),
            Idioma::Polaco => "Hiszpański (Hiszpania)".to_string(),
            Idioma::PortuguesBrasil => "Espanhol (Espanha)".to_string(),
            Idioma::Ruso => "Испанский (Испания)".to_string(),
            Idioma::Sueco => "Spanska (Spanien)".to_string(),
            Idioma::Turco => "İspanyolca (İspanya)".to_string(),
            Idioma::Ucraniano => "Іспанська (Іспанія)".to_string(),
            Idioma::Vietnamita => "Tiếng Tây Ban Nha (Tây Ban Nha)".to_string(),
            Idioma::ChinoSimplificado => "西班牙语（西班牙）".to_string(),
            Idioma::Hungaro => "Spanyol (Spanyolország)".to_string(),
            Idioma::Espanol => "Español (España)".to_string(),
        };
    }

    if codigo == "us" || codigo.starts_with("us,") {
        return match idioma {
            Idioma::Ingles => "English (United States)".to_string(),
            Idioma::BelarusLatino => "Anhłijskaja (Złučanyja Štaty)".to_string(),
            Idioma::Belarus => "Англійская (ЗША)".to_string(),
            Idioma::Catalan => "Anglès (Estats Units)".to_string(),
            Idioma::Checo => "angličtina (Spojené státy americké)".to_string(),
            Idioma::Aleman => "Englisch (USA)".to_string(),
            Idioma::Frances => "Anglais (États-Unis)".to_string(),
            Idioma::Gallego => "Inglés (Estados Unidos)".to_string(),
            Idioma::Italiano => "Inglese (Stati Uniti)".to_string(),
            Idioma::Coreano => "영어(미국)".to_string(),
            Idioma::Kurdo => "Îngilîzî (DYA)".to_string(),
            Idioma::Neerlandes => "Engels (Verenigde Staten)".to_string(),
            Idioma::NoruegoNynorsk => "Engelsk (USA)".to_string(),
            Idioma::Polaco => "angielski (Stany Zjednoczone)".to_string(),
            Idioma::PortuguesBrasil => "Inglês (Estados Unidos)".to_string(),
            Idioma::Ruso => "Английский (США)".to_string(),
            Idioma::Sueco => "Engelska (USA)".to_string(),
            Idioma::Turco => "İngilizce (Amerika Birleşik Devletleri)".to_string(),
            Idioma::Ucraniano => "англійська (Сполучені Штати)".to_string(),
            Idioma::Vietnamita => "Tiếng Anh (Hoa Kỳ)".to_string(),
            Idioma::ChinoSimplificado => "英语（美国）".to_string(),
            Idioma::Hungaro => "Angol (Egyesült Államok)".to_string(),
            Idioma::Espanol => "Inglés (Estados Unidos)".to_string(),
        };
    }

    if codigo == "hu" || codigo.starts_with("hu,") {
        return match idioma {
            Idioma::Ingles => "Hungarian".to_string(),
            Idioma::BelarusLatino => "vjenhjerski".to_string(),
            Idioma::Belarus => "Венгерская".to_string(),
            Idioma::Catalan => "hongarès".to_string(),
            Idioma::Checo => "maďarština".to_string(),
            Idioma::Aleman => "Ungarisch".to_string(),
            Idioma::Frances => "hongrois".to_string(),
            Idioma::Gallego => "húngaro".to_string(),
            Idioma::Italiano => "Ungherese".to_string(),
            Idioma::Coreano => "헝가리어".to_string(),
            Idioma::Kurdo => "Macarî".to_string(),
            Idioma::Neerlandes => "Hongaars".to_string(),
            Idioma::NoruegoNynorsk => "Ungarsk".to_string(),
            Idioma::Polaco => "węgierski".to_string(),
            Idioma::PortuguesBrasil => "Húngaro".to_string(),
            Idioma::Ruso => "Венгерский".to_string(),
            Idioma::Sueco => "Ungerska".to_string(),
            Idioma::Turco => "Macarca".to_string(),
            Idioma::Ucraniano => "Угорська".to_string(),
            Idioma::Vietnamita => "Hungary".to_string(),
            Idioma::ChinoSimplificado => "匈牙利语".to_string(),
            Idioma::Hungaro => "Magyar".to_string(),
            Idioma::Espanol => "Húngaro".to_string(),
        };
    }

    valor.to_string()
}

fn firmware_humano(valor: &str) -> String {
    match valor.to_ascii_lowercase().as_str() {
        "uefi" => "UEFI".to_string(),
        "bios" | "legacy" => "BIOS".to_string(),
        _ => valor.to_string(),
    }
}

fn estado_cuenta_humano(idioma: Idioma, valor: &str) -> String {
    match (idioma, valor.to_ascii_lowercase().as_str()) {
        (Idioma::Ingles, "adopted" | "managed") => "Main user".to_string(),
        (Idioma::BelarusLatino, "adopted" | "managed") => "Asnoŭny karystałnik".to_string(),
        (Idioma::Belarus, "adopted" | "managed") => "Асноўны карыстальнік".to_string(),
        (Idioma::Catalan, "adopted" | "managed") => "Usuari principal".to_string(),
        (Idioma::Checo, "adopted" | "managed") => "Hlavní uživatel".to_string(),
        (Idioma::Aleman, "adopted" | "managed") => "Hauptbenutzer".to_string(),
        (Idioma::Frances, "adopted" | "managed") => "Utilisateur principal".to_string(),
        (Idioma::Gallego, "adopted" | "managed") => "Usuario principal".to_string(),
        (Idioma::Italiano, "adopted" | "managed") => "Utente principale".to_string(),
        (Idioma::Coreano, "adopted" | "managed") => "기본 사용자".to_string(),
        (Idioma::Kurdo, "adopted" | "managed") => "Bikarhênerê sereke".to_string(),
        (Idioma::Neerlandes, "adopted" | "managed") => "Hoofdgebruiker".to_string(),
        (Idioma::NoruegoNynorsk, "adopted" | "managed") => "Hovedbruker".to_string(),
        (Idioma::Polaco, "adopted" | "managed") => "Główny użytkownik".to_string(),
        (Idioma::PortuguesBrasil, "adopted" | "managed") => "Usuário principal".to_string(),
        (Idioma::Ruso, "adopted" | "managed") => "Главный пользователь".to_string(),
        (Idioma::Sueco, "adopted" | "managed") => "Huvudanvändare".to_string(),
        (Idioma::Turco, "adopted" | "managed") => "Ana kullanıcı".to_string(),
        (Idioma::Ucraniano, "adopted" | "managed") => "Головний користувач".to_string(),
        (Idioma::Vietnamita, "adopted" | "managed") => "Người dùng chính".to_string(),
        (Idioma::ChinoSimplificado, "adopted" | "managed") => "主要用户".to_string(),
        (Idioma::Hungaro, "adopted" | "managed") => "Elsődleges felhasználó".to_string(),
        (_, "adopted" | "managed") => "Usuario principal".to_string(),
        _ => valor.to_string(),
    }
}

fn frase_generaciones(idioma: Idioma, conservar: usize, eliminar: usize) -> String {
    match idioma {
        Idioma::Ingles => {
            let guardadas = if conservar == 1 {
                "Keeps 1 version for recovery".to_string()
            } else {
                format!("Keeps {conservar} versions for recovery")
            };

            let borradas = if eliminar == 1 {
                "removes 1 older version".to_string()
            } else {
                format!("removes {eliminar} older versions")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::BelarusLatino => {
            let guardadas = if conservar == 1 {
                "Zachoŭvaje 1 vjersiju dłja adnaŭłjennja".to_string()
            } else {
                format!("Zachoŭvaje {conservar} vjersii dłja adnaŭłjennja")
            };

            let borradas = if eliminar == 1 {
                "vydałjaje 1 staruju vjersiju".to_string()
            } else {
                format!("vydałjaje {eliminar} staryja vjersii")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Belarus => {
            let guardadas = if conservar == 1 {
                "Захоўвае 1 версію для аднаўлення".to_string()
            } else {
                format!("Захоўвае {conservar} версіі для аднаўлення")
            };

            let borradas = if eliminar == 1 {
                "выдаляе 1 старую версію".to_string()
            } else {
                format!("выдаляе {eliminar} старыя версіі")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Catalan => {
            let guardadas = if conservar == 1 {
                "Conserva 1 versió per a la recuperació".to_string()
            } else {
                format!("Manté les versions {conservar} per a la recuperació")
            };

            let borradas = if eliminar == 1 {
                "elimina 1 versió anterior".to_string()
            } else {
                format!("elimina {eliminar} versions anteriors")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Checo => {
            let guardadas = if conservar == 1 {
                "Uchovává 1 verzi pro obnovení".to_string()
            } else {
                format!("Uchovává verze {conservar} pro obnovení")
            };

            let borradas = if eliminar == 1 {
                "odebere 1 starší verzi".to_string()
            } else {
                format!("odstraňuje {eliminar} starší verze")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Aleman => {
            let guardadas = if conservar == 1 {
                "Behält 1 Version zur Wiederherstellung".to_string()
            } else {
                format!("Behält {conservar} Versionen zur Wiederherstellung")
            };

            let borradas = if eliminar == 1 {
                "entfernt 1 ältere Version".to_string()
            } else {
                format!("entfernt {eliminar} ältere Versionen")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Frances => {
            let guardadas = if conservar == 1 {
                "Conserve 1 version pour la récupération".to_string()
            } else {
                format!("Conserve les versions {conservar} pour la récupération")
            };

            let borradas = if eliminar == 1 {
                "supprime 1 ancienne version".to_string()
            } else {
                format!("supprime les anciennes versions de {eliminar}")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Gallego => {
            let guardadas = if conservar == 1 {
                "Mantén 1 versión para a súa recuperación".to_string()
            } else {
                format!("Mantén as versións de {conservar} para a súa recuperación")
            };

            let borradas = if eliminar == 1 {
                "elimina 1 versión anterior".to_string()
            } else {
                format!("elimina {eliminar} versións anteriores")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Italiano => {
            let guardadas = if conservar == 1 {
                "Conserva 1 versione per il ripristino".to_string()
            } else {
                format!("Mantiene le versioni {conservar} per il ripristino")
            };

            let borradas = if eliminar == 1 {
                "rimuove 1 versione precedente".to_string()
            } else {
                format!("rimuove {eliminar} versioni precedenti")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Coreano => {
            let guardadas = if conservar == 1 {
                "복구를 위해 1개 버전 유지".to_string()
            } else {
                format!("복구를 위해 {conservar} 버전을 유지합니다.")
            };

            let borradas = if eliminar == 1 {
                "은 이전 버전 1개를 제거합니다.".to_string()
            } else {
                format!("은 {eliminar} 이전 버전을 제거합니다.")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Kurdo => {
            let guardadas = if conservar == 1 {
                "1 guhertoya ji bo vegirtinê digire".to_string()
            } else {
                format!("Guhertoyên {conservar} ji bo vegirtinê digire")
            };

            let borradas = if eliminar == 1 {
                "1 guhertoya kevntir jê dike".to_string()
            } else {
                format!("{eliminar} guhertoyên kevntir jê dike")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Neerlandes => {
            let guardadas = if conservar == 1 {
                "Behoudt 1 versie voor herstel".to_string()
            } else {
                format!("Behoudt {conservar} versies voor herstel")
            };

            let borradas = if eliminar == 1 {
                "verwijdert 1 oudere versie".to_string()
            } else {
                format!("verwijdert {eliminar} oudere versies")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::NoruegoNynorsk => {
            let guardadas = if conservar == 1 {
                "Beholder 1 versjon for gjenoppretting".to_string()
            } else {
                format!("Beholder {conservar} versjoner for gjenoppretting")
            };

            let borradas = if eliminar == 1 {
                "fjerner 1 eldre versjon".to_string()
            } else {
                format!("fjerner {eliminar} eldre versjoner")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Polaco => {
            let guardadas = if conservar == 1 {
                "Zachowuje 1 wersję do odzyskania".to_string()
            } else {
                format!("Przechowuje wersje {conservar} do odzyskania")
            };

            let borradas = if eliminar == 1 {
                "usuwa 1 starszą wersję".to_string()
            } else {
                format!("usuwa starsze wersje {eliminar}")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::PortuguesBrasil => {
            let guardadas = if conservar == 1 {
                "Mantém 1 versão para recuperação".to_string()
            } else {
                format!("Mantém versões {conservar} para recuperação")
            };

            let borradas = if eliminar == 1 {
                "remove 1 versão mais antiga".to_string()
            } else {
                format!("remove versões mais antigas de {eliminar}")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Ruso => {
            let guardadas = if conservar == 1 {
                "Сохраняет 1 версию для восстановления.".to_string()
            } else {
                format!("Сохраняет версии {conservar} для восстановления.")
            };

            let borradas = if eliminar == 1 {
                "удаляет 1 старую версию".to_string()
            } else {
                format!("удаляет {eliminar} более старые версии")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Sueco => {
            let guardadas = if conservar == 1 {
                "Behåller 1 version för återställning".to_string()
            } else {
                format!("Behåller {conservar} versioner för återställning")
            };

            let borradas = if eliminar == 1 {
                "tar bort 1 äldre version".to_string()
            } else {
                format!("tar bort {eliminar} äldre versioner")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Turco => {
            let guardadas = if conservar == 1 {
                "Kurtarma için 1 sürümü saklar".to_string()
            } else {
                format!("Keeps {conservar} versions for recovery")
            };

            let borradas = if eliminar == 1 {
                "1 eski sürümü kaldırır".to_string()
            } else {
                format!("removes {eliminar} older versions")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Ucraniano => {
            let guardadas = if conservar == 1 {
                "Зберігає 1 версію для відновлення".to_string()
            } else {
                format!("Зберігає {conservar} версії для відновлення")
            };

            let borradas = if eliminar == 1 {
                "видаляє 1 старішу версію".to_string()
            } else {
                format!("видаляє {eliminar} старіші версії")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Vietnamita => {
            let guardadas = if conservar == 1 {
                "Giữ 1 phiên bản để phục hồi".to_string()
            } else {
                format!("Giữ phiên bản {conservar} để khôi phục")
            };

            let borradas = if eliminar == 1 {
                "xóa 1 phiên bản cũ hơn".to_string()
            } else {
                format!("xóa {eliminar} phiên bản cũ hơn")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::ChinoSimplificado => {
            let guardadas = if conservar == 1 {
                "保留 1 个版本用于恢复".to_string()
            } else {
                format!("保留 {conservar} 版本以供恢复")
            };

            let borradas = if eliminar == 1 {
                "删除 1 个旧版本".to_string()
            } else {
                format!("删除 {eliminar} 旧版本")
            };

            format!("{guardadas} · {borradas}")
        }
        Idioma::Hungaro => format!(
            "{conservar} verzió megmarad helyreállításhoz · {eliminar} régebbi verzió törlődik"
        ),
        Idioma::Espanol => {
            let guardadas = if conservar == 1 {
                "Conserva 1 versión para recuperación".to_string()
            } else {
                format!("Conserva {conservar} versiones para recuperación")
            };

            let borradas = if eliminar == 1 {
                "elimina 1 versión antigua".to_string()
            } else {
                format!("elimina {eliminar} versiones antiguas")
            };

            format!("{guardadas} · {borradas}")
        }
    }
}

fn frase_fuentes(idioma: Idioma, cantidad: usize) -> String {
    match idioma {
        Idioma::Ingles => format!(
            "{cantidad} configured components. Refreshes the available versions; it does not install anything yet."
        ),
        Idioma::BelarusLatino => format!(
            "{cantidad} kanfihuravanyja kampanjenty. Abnaŭłjaje dastupnyja vjersii; jon jašče ničoha nje ŭstałjoŭvaje."
        ),
        Idioma::Belarus => format!(
            "{cantidad} канфігураваныя кампаненты. Абнаўляе даступныя версіі; ён яшчэ нічога не ўсталёўвае."
        ),
        Idioma::Catalan => format!(
            "{cantidad} components configurats. Actualitza les versions disponibles; encara no instal·la res."
        ),
        Idioma::Checo => format!(
            "{cantidad} nakonfigurované komponenty. Aktualizuje dostupné verze; zatím nic neinstaluje."
        ),
        Idioma::Aleman => format!(
            "{cantidad} konfigurierte Komponenten. Aktualisiert die verfügbaren Versionen; Es wird noch nichts installiert."
        ),
        Idioma::Frances => format!(
            "{cantidad} composants configurés. Actualise les versions disponibles ; il n'installe encore rien."
        ),
        Idioma::Gallego => format!(
            "{cantidad} compoñentes configurados. Actualiza as versións dispoñibles; aínda non instala nada."
        ),
        Idioma::Italiano => format!(
            "{cantidad} componenti configurati. Aggiorna le versioni disponibili; non installa ancora nulla."
        ),
        Idioma::Coreano => format!(
            "{cantidad} 구성 요소가 구성되었습니다. 사용 가능한 버전을 새로 고칩니다. 아직 아무것도 설치하지 않았습니다."
        ),
        Idioma::Kurdo => format!(
            "{cantidad} hêmanên mîhengkirî. Guhertoyên berdest nû dike; ew hîn tiştek saz nake."
        ),
        Idioma::Neerlandes => format!(
            "{cantidad} geconfigureerde componenten. Vernieuwt de beschikbare versies; er wordt nog niets geïnstalleerd."
        ),
        Idioma::NoruegoNynorsk => format!(
            "{cantidad} konfigurerte komponenter. Oppdaterer tilgjengelige versjoner; den installerer ikke noe ennå."
        ),
        Idioma::Polaco => format!(
            "{cantidad} skonfigurowane komponenty. Odświeża dostępne wersje; jeszcze nic nie instaluje."
        ),
        Idioma::PortuguesBrasil => format!(
            "{cantidad} componentes configurados. Atualiza as versões disponíveis; ainda não instala nada."
        ),
        Idioma::Ruso => format!(
            "{cantidad} настроенные компоненты. Обновляет доступные версии; он еще ничего не устанавливает."
        ),
        Idioma::Sueco => format!(
            "{cantidad} konfigurerade komponenter. Uppdaterar tillgängliga versioner; den installerar inget ännu."
        ),
        Idioma::Turco => format!(
            "{cantidad} configured components. Mevcut sürümleri yeniler; henüz hiçbir şey yüklemiyor."
        ),
        Idioma::Ucraniano => format!(
            "{cantidad} налаштовані компоненти. Оновлює доступні версії; він ще нічого не встановлює."
        ),
        Idioma::Vietnamita => format!(
            "{cantidad} thành phần được định cấu hình. Làm mới các phiên bản có sẵn; nó chưa cài đặt gì cả."
        ),
        Idioma::ChinoSimplificado => format!(
            "{cantidad} 配置组件。刷新可用版本；它还没有安装任何东西。"
        ),
        Idioma::Hungaro => format!(
            "{cantidad} beállított összetevő. Frissíti az elérhető verziókat, de még nem telepít semmit."
        ),
        Idioma::Espanol => format!(
            "{cantidad} componentes configurados. Actualiza las versiones disponibles; todavía no instala nada."
        ),
    }
}

fn frase_confirmar_actualizacion(idioma: Idioma, cantidad: usize) -> String {
    match idioma {
        Idioma::Ingles => format!(
            "Update the software catalog for {cantidad} components now? This may use the network. Nothing will be installed or applied to the system yet."
        ),
        Idioma::BelarusLatino => format!(
            "Abnavic katałoh prahramnaha zabjespjačennja dłja kampanjentaŭ {cantidad}? Heta moža vykarystoŭvac sjetku. Pakuł ničoha nje budzje ŭstanoŭłjena abo prymjenjena da sistemy."
        ),
        Idioma::Belarus => format!(
            "Абнавіць каталог праграмнага забеспячэння для кампанентаў {cantidad}? Гэта можа выкарыстоўваць сетку. Пакуль нічога не будзе ўстаноўлена або прыменена да сістэмы."
        ),
        Idioma::Catalan => format!(
            "Actualitzar ara el catàleg de programari per als components {cantidad}? Això pot utilitzar la xarxa. Encara no s'instal·larà ni s'aplicarà res al sistema."
        ),
        Idioma::Checo => format!(
            "Aktualizovat katalog softwaru pro komponenty {cantidad} nyní? To může používat síť. Do systému se zatím nic neinstaluje ani neaplikuje."
        ),
        Idioma::Aleman => format!(
            "Den Softwarekatalog für {cantidad}-Komponenten jetzt aktualisieren? Dies kann das Netzwerk nutzen. Es wird noch nichts installiert oder auf das System angewendet."
        ),
        Idioma::Frances => format!(
            "Mettre à jour le catalogue de logiciels pour les composants {cantidad} maintenant ? Cela peut utiliser le réseau. Rien ne sera encore installé ou appliqué au système."
        ),
        Idioma::Gallego => format!(
            "Actualizar agora o catálogo de software para os compoñentes {cantidad}? Isto pode usar a rede. Aínda non se instalará nin se aplicará nada ao sistema."
        ),
        Idioma::Italiano => format!(
            "Aggiornare adesso il catalogo software per i componenti {cantidad}? Questo potrebbe utilizzare la rete. Non verrà ancora installato o applicato nulla al sistema."
        ),
        Idioma::Coreano => format!(
            "{cantidad} 구성요소에 대한 소프트웨어 카탈로그를 지금 업데이트하시겠습니까? 네트워크를 사용할 수 있습니다. 아직 시스템에 아무것도 설치되거나 적용되지 않습니다."
        ),
        Idioma::Kurdo => format!(
            "Kataloga nermalavê ya ji bo pêkhateyên {cantidad} niha nûve bike? Ev dibe ku torê bikar bîne. Dê hîn tiştek neyê saz kirin an li pergalê were sepandin."
        ),
        Idioma::Neerlandes => format!(
            "De softwarecatalogus voor {cantidad} componenten nu bijwerken? Deze kan gebruik maken van het netwerk. Er wordt nog niets op het systeem geïnstalleerd of toegepast."
        ),
        Idioma::NoruegoNynorsk => format!(
            "Oppdatere programvarekatalogen for {cantidad} komponenter nå? Dette kan bruke nettverket. Ingenting vil bli installert eller brukt på systemet ennå."
        ),
        Idioma::Polaco => format!(
            "Zaktualizować teraz katalog oprogramowania dla komponentów {cantidad}? To może korzystać z sieci. Nic nie zostanie jeszcze zainstalowane ani zastosowane w systemie."
        ),
        Idioma::PortuguesBrasil => format!(
            "Atualizar o catálogo de software para componentes {cantidad} agora? Isso pode usar a rede. Nada será instalado ou aplicado ao sistema ainda."
        ),
        Idioma::Ruso => format!(
            "Обновить каталог программного обеспечения для компонентов {cantidad} сейчас? Это может использовать сеть. Пока ничего не будет установлено или применено в системе."
        ),
        Idioma::Sueco => format!(
            "Uppdatera programvarukatalogen för {cantidad} komponenter nu? Detta kan använda nätverket. Inget kommer att installeras eller tillämpas på systemet ännu."
        ),
        Idioma::Turco => format!(
            "Update the software catalog for {cantidad} components now? Bu ağı kullanabilir. Nothing will be installed or applied to the system yet."
        ),
        Idioma::Ucraniano => format!(
            "Оновити каталог програмного забезпечення для компонентів {cantidad}? Це може використовувати мережу. Ще нічого не буде встановлено або застосовано до системи."
        ),
        Idioma::Vietnamita => format!(
            "Cập nhật danh mục phần mềm cho các thành phần {cantidad} ngay bây giờ? Điều này có thể sử dụng mạng. Sẽ không có gì được cài đặt hoặc áp dụng cho hệ thống."
        ),
        Idioma::ChinoSimplificado => format!(
            "现在更新 {cantidad} 组件的软件目录吗？这可能会使用网络。尚未向系统安装或应用任何内容。"
        ),
        Idioma::Hungaro => format!(
            "Frissítsem most a szoftverkatalógust {cantidad} összetevőhöz? Ez használhatja a hálózatot. Még semmi sem lesz telepítve vagy alkalmazva a rendszeren."
        ),
        Idioma::Espanol => format!(
            "¿Actualizar ahora el catálogo de software para {cantidad} componentes? Puede usar la red. Todavía no se instalará ni aplicará nada al sistema."
        ),
    }
}

fn frase_confirmar_limpieza(idioma: Idioma, eliminar: usize, agresiva: bool) -> String {
    match (idioma, agresiva) {
        (Idioma::Ingles, false) => format!(
            "Remove {eliminar} older system versions? Korunix will keep the versions marked for recovery and free the space they no longer need."
        ),
        (Idioma::BelarusLatino, false) => format!(
            "Vydałic {eliminar} staryja vjersii sistemy? Korunix zachavaje vjersii, paznačanyja dłja adnaŭłjennja, i vyzvałic mjesca, jakoje im bołš nje patrebna."
        ),
        (Idioma::Belarus, false) => format!(
            "Выдаліць {eliminar} старыя версіі сістэмы? Korunix захавае версіі, пазначаныя для аднаўлення, і вызваліць месца, якое ім больш не патрэбна."
        ),
        (Idioma::Catalan, false) => format!(
            "Eliminar {eliminar} versions anteriors del sistema? Korunix mantindrà les versions marcades per a la recuperació i alliberarà l'espai que ja no necessiten."
        ),
        (Idioma::Checo, false) => format!(
            "Odebrat {eliminar} starší verze systému? Korunix zachová verze označené pro obnovu a uvolní místo, které již nepotřebují."
        ),
        (Idioma::Aleman, false) => format!(
            "{eliminar} ältere Systemversionen entfernen? Korunix behält die zur Wiederherstellung markierten Versionen bei und gibt den nicht mehr benötigten Speicherplatz frei."
        ),
        (Idioma::Frances, false) => format!(
            "Supprimer {eliminar} les anciennes versions du système ? Korunix conservera les versions marquées pour la récupération et libérera l'espace dont elles n'ont plus besoin."
        ),
        (Idioma::Gallego, false) => format!(
            "Quitar {eliminar} versións anteriores do sistema? Korunix manterá as versións marcadas para a súa recuperación e liberará o espazo que xa non necesitan."
        ),
        (Idioma::Italiano, false) => format!(
            "Rimuovere {eliminar} versioni precedenti del sistema? Korunix manterrà le versioni contrassegnate per il ripristino e libererà lo spazio di cui non hanno più bisogno."
        ),
        (Idioma::Coreano, false) => format!(
            "{eliminar} 이전 시스템 버전을 제거하시겠습니까? Korunix는 복구용으로 표시된 버전을 유지하고 더 이상 필요하지 않은 공간을 확보합니다."
        ),
        (Idioma::Kurdo, false) => format!(
            "{eliminar} guhertoyên kevintir ên pergalê rakin? Korunix dê guhertoyên ku ji bo başbûnê hatine nîşankirin bihêle û cîhê ku ew êdî ne hewce ye azad bike."
        ),
        (Idioma::Neerlandes, false) => format!(
            "{eliminar} oudere systeemversies verwijderen? Korunix zal de versies gemarkeerd voor herstel behouden en de ruimte vrijmaken die ze niet langer nodig hebben."
        ),
        (Idioma::NoruegoNynorsk, false) => format!(
            "Vil du fjerne {eliminar} eldre systemversjoner? Korunix vil beholde versjonene merket for gjenoppretting og frigjøre plassen de ikke lenger trenger."
        ),
        (Idioma::Polaco, false) => format!(
            "Usunąć {eliminar} starsze wersje systemu? Korunix zachowa wersje oznaczone do odzyskania i zwolni miejsce, którego już nie potrzebują."
        ),
        (Idioma::PortuguesBrasil, false) => format!(
            "Remover versões mais antigas do sistema {eliminar}? Korunix manterá as versões marcadas para recuperação e liberará o espaço que elas não precisam mais."
        ),
        (Idioma::Ruso, false) => format!(
            "Удалить {eliminar} более старые версии системы? Korunix сохранит версии, помеченные для восстановления, и освободит место, которое им больше не нужно."
        ),
        (Idioma::Sueco, false) => format!(
            "Ta bort {eliminar} äldre systemversioner? Korunix kommer att behålla versionerna markerade för återställning och frigöra det utrymme de inte längre behöver."
        ),
        (Idioma::Turco, false) => format!(
            "Remove {eliminar} older system versions? Korunix will keep the versions marked for recovery and free the space they no longer need."
        ),
        (Idioma::Ucraniano, false) => format!(
            "Видалити {eliminar} старіші версії системи? Korunix збереже версії, позначені для відновлення, і звільнить місце, яке їм більше не потрібне."
        ),
        (Idioma::Vietnamita, false) => format!(
            "Xóa {eliminar} phiên bản hệ thống cũ hơn? Korunix sẽ đánh dấu các phiên bản để khôi phục và giải phóng dung lượng mà chúng không còn cần nữa."
        ),
        (Idioma::ChinoSimplificado, false) => format!(
            "删除 {eliminar} 旧系统版本？ Korunix 将保留标记为恢复的版本并释放它们不再需要的空间。"
        ),
        (Idioma::Ingles, true) => format!(
            "Remove {eliminar} older system versions and keep only the versions required for startup and recovery? This frees more space and keeps fewer recovery options."
        ),
        (Idioma::BelarusLatino, true) => format!(
            "Vydałic {eliminar} starych vjersij sistemy i zachavac tołki vjersii, njeabchodnyja dłja zapusku i adnaŭłjennja? Heta vyzvałjaje bołš mjesca i zachoŭvaje mjenš varyjantaŭ adnaŭłjennja."
        ),
        (Idioma::Belarus, true) => format!(
            "Выдаліць {eliminar} старыя версіі сістэмы і захаваць толькі версіі, неабходныя для запуску і аднаўлення? Гэта вызваляе больш месца і захоўвае менш варыянтаў аднаўлення."
        ),
        (Idioma::Catalan, true) => format!(
            "Eliminar {eliminar} les versions anteriors del sistema i mantenir només les versions necessàries per a l'inici i la recuperació? Això allibera més espai i manté menys opcions de recuperació."
        ),
        (Idioma::Checo, true) => format!(
            "Odebrat {eliminar} starší verze systému a ponechat pouze verze potřebné pro spuštění a obnovení? To uvolní více místa a zachová méně možností obnovení."
        ),
        (Idioma::Aleman, true) => format!(
            "{eliminar} ältere Systemversionen entfernen und nur die Versionen behalten, die für den Start und die Wiederherstellung erforderlich sind? Dadurch wird mehr Speicherplatz frei und es stehen weniger Wiederherstellungsoptionen zur Verfügung."
        ),
        (Idioma::Frances, true) => format!(
            "Supprimer {eliminar} les anciennes versions du système et conserver uniquement les versions requises pour le démarrage et la récupération ? Cela libère plus d'espace et conserve moins d'options de récupération."
        ),
        (Idioma::Gallego, true) => format!(
            "Eliminar {eliminar} versións antigas do sistema e manter só as versións necesarias para o inicio e a recuperación? Isto libera máis espazo e mantén menos opcións de recuperación."
        ),
        (Idioma::Italiano, true) => format!(
            "Rimuovere le versioni precedenti del sistema {eliminar} e mantenere solo le versioni richieste per l'avvio e il ripristino? Ciò libera più spazio e mantiene meno opzioni di ripristino."
        ),
        (Idioma::Coreano, true) => format!(
            "{eliminar} 이전 시스템 버전을 제거하고 시작 및 복구에 필요한 버전만 유지하시겠습니까? 이렇게 하면 더 많은 공간이 확보되고 복구 옵션이 더 적게 유지됩니다."
        ),
        (Idioma::Kurdo, true) => format!(
            "{eliminar} guhertoyên kevintir ên pergalê rakin û tenê guhertoyên ku ji bo destpêk û vegerandinê hewce ne bihêlin? Ev bêtir cîhê azad dike û kêmtir vebijarkên başbûnê digire."
        ),
        (Idioma::Neerlandes, true) => format!(
            "{eliminar} oudere systeemversies verwijderen en alleen de versies behouden die nodig zijn voor opstarten en herstel? Hierdoor komt er meer ruimte vrij en blijven er minder herstelopties over."
        ),
        (Idioma::NoruegoNynorsk, true) => format!(
            "Vil du fjerne {eliminar} eldre systemversjoner og beholde bare versjonene som kreves for oppstart og gjenoppretting? Dette frigjør mer plass og beholder færre gjenopprettingsalternativer."
        ),
        (Idioma::Polaco, true) => format!(
            "Usunąć {eliminar} starsze wersje systemu i zachować tylko wersje wymagane do uruchomienia i odzyskiwania? Dzięki temu można zwolnić więcej miejsca i zachować mniej opcji odzyskiwania."
        ),
        (Idioma::PortuguesBrasil, true) => format!(
            "Remover versões mais antigas do sistema {eliminar} e manter apenas as versões necessárias para inicialização e recuperação? Isso libera mais espaço e mantém menos opções de recuperação."
        ),
        (Idioma::Ruso, true) => format!(
            "Удалить {eliminar} старые версии системы и оставить только те версии, которые необходимы для запуска и восстановления? Это освобождает больше места и оставляет меньше вариантов восстановления."
        ),
        (Idioma::Sueco, true) => format!(
            "Ta bort {eliminar} äldre systemversioner och behålla endast de versioner som krävs för start och återställning? Detta frigör mer utrymme och behåller färre återställningsalternativ."
        ),
        (Idioma::Turco, true) => format!(
            "Remove {eliminar} older system versions and keep only the versions required for startup and recovery? This frees more space and keeps fewer recovery options."
        ),
        (Idioma::Ucraniano, true) => format!(
            "Видалити {eliminar} старіші версії системи та зберегти лише версії, необхідні для запуску та відновлення? Це звільняє більше місця та зберігає менше варіантів відновлення."
        ),
        (Idioma::Vietnamita, true) => format!(
            "Xóa {eliminar} phiên bản hệ thống cũ hơn và chỉ giữ lại các phiên bản cần thiết để khởi động và khôi phục? Điều này giải phóng nhiều không gian hơn và giữ ít tùy chọn khôi phục hơn."
        ),
        (Idioma::ChinoSimplificado, true) => format!(
            "删除 {eliminar} 旧系统版本并仅保留启动和恢复所需的版本？这可以释放更多空间并保留更少的恢复选项。"
        ),
        (Idioma::Hungaro, false) => format!(
            "Töröljek {eliminar} régebbi rendszerverziót? A Korunix megtartja a helyreállításhoz jelölt verziókat, és felszabadítja a már nem szükséges helyet."
        ),
        (Idioma::Hungaro, true) => format!(
            "Töröljek {eliminar} régebbi rendszerverziót, és csak az indításhoz és helyreállításhoz szükségeseket tartsam meg? Ez több helyet szabadít fel, de kevesebb helyreállítási lehetőséget hagy."
        ),
        (Idioma::Espanol, false) => format!(
            "¿Eliminar {eliminar} versiones antiguas del sistema? Korunix conservará las versiones marcadas para recuperación y liberará el espacio que ya no necesitan."
        ),
        (Idioma::Espanol, true) => format!(
            "¿Eliminar {eliminar} versiones antiguas y conservar solo las necesarias para iniciar y recuperar el sistema? Libera más espacio, pero deja menos opciones de recuperación."
        ),
    }
}

fn frase_confirmar_recuperacion(idioma: Idioma, _id: u32) -> String {
    match idioma {
        Idioma::Ingles => "Try the selected version only on the next restart? The version that normally starts will not be replaced.".to_string(),
        Idioma::BelarusLatino => "Pasprabavac vybranuju vjersiju tołki pry nastupnym pjerazapusku? Vjersija, jakaja zvyčajna zapuskajecca, nje budzje zamjenjena.".to_string(),
        Idioma::Belarus => "Паспрабаваць выбраную версію толькі пры наступным перазапуску? Версія, якая звычайна запускаецца, не будзе заменена.".to_string(),
        Idioma::Catalan => "Voleu provar la versió seleccionada només al proper reinici? La versió que s'inicia normalment no es substituirà.".to_string(),
        Idioma::Checo => "Vyzkoušet vybranou verzi pouze při příštím restartu? Verze, která se normálně spouští, nebude nahrazena.".to_string(),
        Idioma::Aleman => "Die ausgewählte Version erst beim nächsten Neustart testen? Die normalerweise gestartete Version wird nicht ersetzt.".to_string(),
        Idioma::Frances => "Essayer la version sélectionnée uniquement au prochain redémarrage ? La version qui démarre normalement ne sera pas remplacée.".to_string(),
        Idioma::Gallego => "Probar a versión seleccionada só no seguinte reinicio? A versión que se inicia normalmente non será substituída.".to_string(),
        Idioma::Italiano => "Provare la versione selezionata solo al prossimo riavvio? La versione che normalmente si avvia non verrà sostituita.".to_string(),
        Idioma::Coreano => "다음에 다시 시작할 때만 선택한 버전을 사용해 보시겠습니까? 정상적으로 시작되는 버전은 교체되지 않습니다.".to_string(),
        Idioma::Kurdo => "Tenê di destpêkirina nû de guhertoya hilbijartî biceribîne? Guhertoya ku bi gelemperî dest pê dike dê neyê guheztin.".to_string(),
        Idioma::Neerlandes => "Probeer de geselecteerde versie alleen bij de volgende herstart? De versie die normaal start, wordt niet vervangen.".to_string(),
        Idioma::NoruegoNynorsk => "Prøv den valgte versjonen bare ved neste omstart? Versjonen som normalt starter vil ikke bli erstattet.".to_string(),
        Idioma::Polaco => "Wypróbować wybraną wersję dopiero przy następnym uruchomieniu? Wersja, która normalnie się uruchamia, nie zostanie zastąpiona.".to_string(),
        Idioma::PortuguesBrasil => "Tentar a versão selecionada somente na próxima reinicialização? A versão que normalmente inicia não será substituída.".to_string(),
        Idioma::Ruso => "Попробовать выбранную версию только при следующем перезапуске? Версия, которая нормально запускается, не будет заменена.".to_string(),
        Idioma::Sueco => "Prova endast den valda versionen vid nästa omstart? Den version som normalt startar kommer inte att ersättas.".to_string(),
        Idioma::Turco => "Try the selected version only on the next restart? The version that normally starts will not be replaced.".to_string(),
        Idioma::Ucraniano => "Спробувати вибрану версію лише під час наступного перезавантаження? Версія, яка зазвичай запускається, не буде замінена.".to_string(),
        Idioma::Vietnamita => "Chỉ thử phiên bản đã chọn trong lần khởi động lại tiếp theo? Phiên bản thường khởi động sẽ không được thay thế.".to_string(),
        Idioma::ChinoSimplificado => "仅在下次重新启动时尝试所选版本？正常启动的版本不会被替换。".to_string(),
        Idioma::Hungaro => "Csak a következő újraindításkor próbáljam ki a kiválasztott verziót? A rendszerint induló verzió nem változik.".to_string(),
        Idioma::Espanol => "¿Probar la versión seleccionada solo en el próximo reinicio? La versión que inicia normalmente no será reemplazada.".to_string(),
    }
}

fn frase_confirmar_expulsion(idioma: Idioma, _dispositivo: &str, pesada: bool) -> String {
    match (idioma, pesada) {
        (Idioma::Ingles, true) => "Eject this drive safely? Korunix will first wait until all pending data has finished saving.".to_string(),
        (Idioma::BelarusLatino, true) => "Bjaspječna vynjac hety dysk? Korunix spačatku pačakaje, pakuł nje zavjeršycca zachavannje ŭsich čakajučych danych.".to_string(),
        (Idioma::Belarus, true) => "Бяспечна выняць гэты дыск? Korunix спачатку пачакае, пакуль не завершыцца захаванне ўсіх чакаючых даных.".to_string(),
        (Idioma::Catalan, true) => "Expulsar aquesta unitat de manera segura? Korunix esperarà primer fins que totes les dades pendents s'acabin de desar.".to_string(),
        (Idioma::Checo, true) => "Bezpečně vysunout tento disk? Korunix nejprve počká, dokud se všechna čekající data neuloží.".to_string(),
        (Idioma::Aleman, true) => "Dieses Laufwerk sicher auswerfen? Korunix wartet zunächst, bis alle ausstehenden Daten gespeichert wurden.".to_string(),
        (Idioma::Frances, true) => "Éjecter ce disque en toute sécurité ? Korunix attendra d'abord que toutes les données en attente aient fini d'être enregistrées.".to_string(),
        (Idioma::Gallego, true) => "Expulsar esta unidade de forma segura? Korunix agardará primeiro ata que rematen de gardar todos os datos pendentes.".to_string(),
        (Idioma::Italiano, true) => "Espellere questa unità in modo sicuro? Korunix attenderà innanzitutto che tutti i dati in sospeso abbiano terminato il salvataggio.".to_string(),
        (Idioma::Coreano, true) => "이 드라이브를 안전하게 꺼내시겠습니까? Korunix는 먼저 보류 중인 모든 데이터 저장이 완료될 때까지 기다립니다.".to_string(),
        (Idioma::Kurdo, true) => "Vê ajokerê bi ewlehî derxe? Korunix dê pêşî li bendê bimîne heya ku hemî daneyên li bendê xilas bibin.".to_string(),
        (Idioma::Neerlandes, true) => "Deze schijf veilig uitwerpen? Korunix wacht eerst totdat alle openstaande gegevens zijn opgeslagen.".to_string(),
        (Idioma::NoruegoNynorsk, true) => "Ta ut denne stasjonen trygt? Korunix vil først vente til alle ventende data er ferdig lagret.".to_string(),
        (Idioma::Polaco, true) => "Bezpiecznie wysunąć ten dysk? Korunix najpierw poczeka, aż wszystkie oczekujące dane zostaną zakończone.".to_string(),
        (Idioma::PortuguesBrasil, true) => "Ejetar esta unidade com segurança? Korunix irá primeiro esperar até que todos os dados pendentes terminem de ser salvos.".to_string(),
        (Idioma::Ruso, true) => "Безопасно извлечь этот диск? Korunix сначала дождется завершения сохранения всех ожидающих данных.".to_string(),
        (Idioma::Sueco, true) => "Mata ut den här enheten på ett säkert sätt? Korunix kommer först att vänta tills all väntande data har sparats.".to_string(),
        (Idioma::Turco, true) => "Bu sürücü güvenli bir şekilde çıkarılsın mı? Korunix will first wait until all pending data has finished saving.".to_string(),
        (Idioma::Ucraniano, true) => "Безпечно вийняти цей диск? Korunix спочатку зачекає, доки завершиться збереження всіх незавершених даних.".to_string(),
        (Idioma::Vietnamita, true) => "Đẩy ổ đĩa này ra một cách an toàn? Đầu tiên Korunix sẽ đợi cho đến khi tất cả dữ liệu đang chờ xử lý được lưu xong.".to_string(),
        (Idioma::ChinoSimplificado, true) => "安全弹出此驱动器吗？ Korunix 将首先等待所有待处理数据完成保存。".to_string(),
        (Idioma::Ingles, false) => "Eject this drive safely now?".to_string(),
        (Idioma::BelarusLatino, false) => "Chočacje bjaspječna vynjac hety dysk zaraz?".to_string(),
        (Idioma::Belarus, false) => "Бяспечна выняць гэты дыск?".to_string(),
        (Idioma::Catalan, false) => "Expulsar aquesta unitat de manera segura ara?".to_string(),
        (Idioma::Checo, false) => "Vysunout tento disk nyní bezpečně?".to_string(),
        (Idioma::Aleman, false) => "Dieses Laufwerk jetzt sicher auswerfen?".to_string(),
        (Idioma::Frances, false) => "Éjecter ce disque en toute sécurité maintenant ?".to_string(),
        (Idioma::Gallego, false) => "Expulsar esta unidade de forma segura agora?".to_string(),
        (Idioma::Italiano, false) => "Espellere questa unità in modo sicuro adesso?".to_string(),
        (Idioma::Coreano, false) => "지금 이 드라이브를 안전하게 꺼내시겠습니까?".to_string(),
        (Idioma::Kurdo, false) => "Vê ajokerê niha bi ewlehî derxe?".to_string(),
        (Idioma::Neerlandes, false) => "Deze schijf nu veilig uitwerpen?".to_string(),
        (Idioma::NoruegoNynorsk, false) => "Ta ut denne stasjonen trygt nå?".to_string(),
        (Idioma::Polaco, false) => "Czy teraz bezpiecznie wysunąć ten dysk?".to_string(),
        (Idioma::PortuguesBrasil, false) => "Ejetar esta unidade com segurança agora?".to_string(),
        (Idioma::Ruso, false) => "Безопасно извлечь этот диск?".to_string(),
        (Idioma::Sueco, false) => "Mata ut den här enheten säkert nu?".to_string(),
        (Idioma::Turco, false) => "Eject this drive safely now?".to_string(),
        (Idioma::Ucraniano, false) => "Безпечно вийняти цей диск?".to_string(),
        (Idioma::Vietnamita, false) => "Đẩy ổ đĩa này ra một cách an toàn ngay bây giờ?".to_string(),
        (Idioma::ChinoSimplificado, false) => "现在安全弹出该驱动器吗？".to_string(),
        (Idioma::Hungaro, true) => "Biztonságosan leválasszam ezt a meghajtót? A Korunix előbb megvárja, amíg minden függőben lévő adat mentése befejeződik.".to_string(),
        (Idioma::Hungaro, false) => "Biztonságosan leválasszam most ezt a meghajtót?".to_string(),
        (Idioma::Espanol, true) => "¿Expulsar esta unidad de forma segura? Korunix esperará primero a que terminen de guardarse todos los datos pendientes.".to_string(),
        (Idioma::Espanol, false) => "¿Expulsar esta unidad de forma segura ahora?".to_string(),
    }
}

fn frase_confirmar_firmware(idioma: Idioma, nombre: &str, efecto: &str) -> String {
    let efecto_humano = match (idioma, efecto) {
        (Idioma::Ingles, "reboot") => "A restart will be required.",
        (Idioma::BelarusLatino, "reboot") => "Spatrebicca pjerazapusk.",
        (Idioma::Belarus, "reboot") => "Спатрэбіцца перазапуск.",
        (Idioma::Catalan, "reboot") => "Es requerirà un reinici.",
        (Idioma::Checo, "reboot") => "Bude vyžadován restart.",
        (Idioma::Aleman, "reboot") => "Ein Neustart ist erforderlich.",
        (Idioma::Frances, "reboot") => "Un redémarrage sera nécessaire.",
        (Idioma::Gallego, "reboot") => "Será necesario un reinicio.",
        (Idioma::Italiano, "reboot") => "Sarà necessario un riavvio.",
        (Idioma::Coreano, "reboot") => "다시 시작해야 합니다.",
        (Idioma::Kurdo, "reboot") => "Ji nû ve destpêkek pêdivî ye.",
        (Idioma::Neerlandes, "reboot") => "Een herstart is vereist.",
        (Idioma::NoruegoNynorsk, "reboot") => "En omstart vil være nødvendig.",
        (Idioma::Polaco, "reboot") => "Wymagane będzie ponowne uruchomienie.",
        (Idioma::PortuguesBrasil, "reboot") => "Será necessária uma reinicialização.",
        (Idioma::Ruso, "reboot") => "Потребуется перезагрузка.",
        (Idioma::Sueco, "reboot") => "En omstart kommer att krävas.",
        (Idioma::Turco, "reboot") => "A restart will be required.",
        (Idioma::Ucraniano, "reboot") => "Буде потрібно перезавантаження.",
        (Idioma::Vietnamita, "reboot") => "Sẽ cần phải khởi động lại.",
        (Idioma::ChinoSimplificado, "reboot") => "需要重新启动。",
        (Idioma::Ingles, "shutdown") => "A full shutdown will be required.",
        (Idioma::BelarusLatino, "shutdown") => "Spatrebicca poŭnaje adkłjučennje.",
        (Idioma::Belarus, "shutdown") => "Спатрэбіцца поўнае адключэнне.",
        (Idioma::Catalan, "shutdown") => "Es requerirà un tancament complet.",
        (Idioma::Checo, "shutdown") => "Bude vyžadováno úplné vypnutí.",
        (Idioma::Aleman, "shutdown") => "Ein vollständiges Herunterfahren ist erforderlich.",
        (Idioma::Frances, "shutdown") => "Un arrêt complet sera nécessaire.",
        (Idioma::Gallego, "shutdown") => "Será necesario un apagado completo.",
        (Idioma::Italiano, "shutdown") => "Sarà necessario uno spegnimento completo.",
        (Idioma::Coreano, "shutdown") => "전체 종료가 필요합니다.",
        (Idioma::Kurdo, "shutdown") => "Girtinek tam dê hewce be.",
        (Idioma::Neerlandes, "shutdown") => "Een volledige uitschakeling is vereist.",
        (Idioma::NoruegoNynorsk, "shutdown") => "En fullstendig avstengning vil være nødvendig.",
        (Idioma::Polaco, "shutdown") => "Wymagane będzie pełne wyłączenie.",
        (Idioma::PortuguesBrasil, "shutdown") => "Será necessário um desligamento completo.",
        (Idioma::Ruso, "shutdown") => "Потребуется полное выключение.",
        (Idioma::Sueco, "shutdown") => "En fullständig avstängning kommer att krävas.",
        (Idioma::Turco, "shutdown") => "A full shutdown will be required.",
        (Idioma::Ucraniano, "shutdown") => "Буде потрібно повне завершення роботи.",
        (Idioma::Vietnamita, "shutdown") => "Cần phải tắt máy hoàn toàn.",
        (Idioma::ChinoSimplificado, "shutdown") => "需要完全关闭。",
        (Idioma::Hungaro, "reboot") => "Újraindítás szükséges.",
        (Idioma::Hungaro, "shutdown") => "Teljes leállítás szükséges.",
        (_, "reboot") => "Será necesario reiniciar.",
        (_, "shutdown") => "Será necesario apagar completamente el equipo.",
        (Idioma::Ingles, _) => "The effect is immediate.",
        (Idioma::BelarusLatino, _) => "Efjekt adčuvałny adrazu.",
        (Idioma::Belarus, _) => "Эфект імгненны.",
        (Idioma::Catalan, _) => "L'efecte és immediat.",
        (Idioma::Checo, _) => "Účinek je okamžitý.",
        (Idioma::Aleman, _) => "Die Wirkung tritt sofort ein.",
        (Idioma::Frances, _) => "L'effet est immédiat.",
        (Idioma::Gallego, _) => "O efecto é inmediato.",
        (Idioma::Italiano, _) => "L'effetto è immediato.",
        (Idioma::Coreano, _) => "효과는 즉각적입니다.",
        (Idioma::Kurdo, _) => "Bandora yekser e.",
        (Idioma::Neerlandes, _) => "Het effect treedt onmiddellijk op.",
        (Idioma::NoruegoNynorsk, _) => "Effekten er umiddelbar.",
        (Idioma::Polaco, _) => "Efekt jest natychmiastowy.",
        (Idioma::PortuguesBrasil, _) => "O efeito é imediato.",
        (Idioma::Ruso, _) => "Эффект немедленный.",
        (Idioma::Sueco, _) => "Effekten är omedelbar.",
        (Idioma::Turco, _) => "Etkisi anında görülür.",
        (Idioma::Ucraniano, _) => "Ефект миттєвий.",
        (Idioma::Vietnamita, _) => "Hiệu quả tức thì.",
        (Idioma::ChinoSimplificado, _) => "效果是立竿见影的。",
        (Idioma::Hungaro, _) => "A hatás azonnali.",
        (_, _) => "El efecto es inmediato.",
    };

    match idioma {
        Idioma::Ingles => format!("Install the firmware update for {nombre}? {efecto_humano} Korunix will not restart or shut down automatically."),
        Idioma::BelarusLatino => format!("Ustałjavac abnaŭłjennje prašyŭki dłja {nombre}? {efecto_humano} Korunix nje budzje pjerazahružacca abo vykłjučacca aŭtamatyčna."),
        Idioma::Belarus => format!("Усталяваць абнаўленне прашыўкі для {nombre}? {efecto_humano} Korunix не будзе перазагружацца або выключацца аўтаматычна."),
        Idioma::Catalan => format!("Instal·lar l'actualització del microprogramari per a {nombre}? {efecto_humano} Korunix no es reiniciarà ni s'apagarà automàticament."),
        Idioma::Checo => format!("Nainstalovat aktualizaci firmwaru pro {nombre}? {efecto_humano} Korunix se automaticky nerestartuje ani nevypne."),
        Idioma::Aleman => format!("Firmware-Update für {nombre} installieren? {efecto_humano} Korunix wird nicht automatisch neu gestartet oder heruntergefahren."),
        Idioma::Frances => format!("Installer la mise à jour du micrologiciel pour {nombre} ? {efecto_humano} Korunix ne redémarrera pas ou ne s'arrêtera pas automatiquement."),
        Idioma::Gallego => format!("Instalar a actualización de firmware para {nombre}? {efecto_humano} Korunix non se reiniciará nin se apagará automaticamente."),
        Idioma::Italiano => format!("Installare l'aggiornamento firmware per {nombre}? {efecto_humano} Korunix non si riavvierà o si spegnerà automaticamente."),
        Idioma::Coreano => format!("{nombre}용 펌웨어 업데이트를 설치하시겠습니까? {efecto_humano} Korunix는 자동으로 다시 시작되거나 종료되지 않습니다."),
        Idioma::Kurdo => format!("Nûvekirina firmware ji bo {nombre} saz bike? {efecto_humano} Korunix ji nû ve dest pê nake an bixweber nayê girtin."),
        Idioma::Neerlandes => format!("De firmware-update voor {nombre} installeren? {efecto_humano} Kounix zal niet automatisch opnieuw opstarten of afsluiten."),
        Idioma::NoruegoNynorsk => format!("Installere fastvareoppdateringen for {nombre}? {efecto_humano} Korunix vil ikke starte på nytt eller slå seg av automatisk."),
        Idioma::Polaco => format!("Zainstalować aktualizację oprogramowania sprzętowego dla {nombre}? {efecto_humano} Korunix nie uruchomi się ponownie ani nie wyłączy się automatycznie."),
        Idioma::PortuguesBrasil => format!("Instalar a atualização de firmware para {nombre}? {efecto_humano} Korunix não irá reiniciar ou desligar automaticamente."),
        Idioma::Ruso => format!("Установить обновление прошивки для {nombre}? {efecto_humano} Korunix не перезагружается и не выключается автоматически."),
        Idioma::Sueco => format!("Installera firmwareuppdateringen för {nombre}? {efecto_humano} Korunix kommer inte att starta om eller stängas av automatiskt."),
        Idioma::Turco => format!("Install the firmware update for {nombre}? {efecto_humano} Korunix will not restart or shut down automatically."),
        Idioma::Ucraniano => format!("Установити оновлення мікропрограми для {nombre}? {efecto_humano} Korunix не перезавантажиться або вимкнеться автоматично."),
        Idioma::Vietnamita => format!("Cài đặt bản cập nhật chương trình cơ sở cho {nombre}? {efecto_humano} Korunix sẽ không tự động khởi động lại hoặc tắt."),
        Idioma::ChinoSimplificado => format!("安装 {nombre} 的固件更新吗？ {efecto_humano} Korunix 不会自动重新启动或关闭。"),
        Idioma::Hungaro => format!("Telepítsem a firmware-frissítést ehhez: {nombre}? {efecto_humano} A Korunix nem indítja újra és nem állítja le automatikusan a gépet."),
        Idioma::Espanol => format!("¿Instalar la actualización de firmware para {nombre}? {efecto_humano} Korunix no reiniciará ni apagará el equipo automáticamente."),
    }
}

fn parece_nombre_tecnico_audio(valor: &str) -> bool {
    let valor = valor.to_ascii_lowercase();
    valor.contains("controller")
        || valor.contains("analog stereo")
        || valor.contains("digital stereo")
        || valor.contains("hdmi / displayport")
        || valor.contains("hdmi/displayport")
        || valor.starts_with("alsa_")
        || valor.starts_with("pci-")
}

fn descripcion_pantalla_audio(
    idioma: Idioma,
    audio: &Value,
    item: Option<&Value>,
    respaldo: &str,
) -> Option<String> {
    let descripcion = item
        .and_then(|item| item.get("description"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let puerto = item
        .and_then(|item| item.get("activePort"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let huella = format!("{respaldo} {descripcion}").to_ascii_lowercase();
    let es_pantalla = puerto.starts_with("hdmi-output-")
        || huella.contains("hdmi")
        || huella.contains("displayport");

    if !es_pantalla {
        return None;
    }

    let pantallas = audio
        .get("displayConnections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let coincidencias = pantallas
        .iter()
        .filter(|pantalla| {
            pantalla
                .get("monitorName")
                .and_then(Value::as_str)
                .map(|nombre| huella.contains(&nombre.to_ascii_lowercase()))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    let pantalla = if coincidencias.len() == 1 {
        coincidencias.first().copied()
    } else if pantallas.len() == 1 {
        pantallas.first()
    } else {
        None
    };

    let Some(pantalla) = pantalla else {
        return Some(match idioma {
            Idioma::Ingles => "External display · Connection not identified".to_string(),
            Idioma::BelarusLatino => "Znješni dyspłjej · Złučennje nje vyznačana".to_string(),
            Idioma::Belarus => "Знешні дысплей · Злучэнне не вызначана".to_string(),
            Idioma::Catalan => "Pantalla externa · Connexió no identificada".to_string(),
            Idioma::Checo => "Externí displej · Připojení nebylo identifikováno".to_string(),
            Idioma::Aleman => "Externes Display · Verbindung nicht identifiziert".to_string(),
            Idioma::Frances => "Affichage externe · Connexion non identifiée".to_string(),
            Idioma::Gallego => "Pantalla externa · Conexión non identificada".to_string(),
            Idioma::Italiano => "Display esterno · Connessione non identificata".to_string(),
            Idioma::Coreano => "외부 디스플레이 · 연결이 식별되지 않음".to_string(),
            Idioma::Kurdo => "Nîşandana derve · Têkilî nehate naskirin".to_string(),
            Idioma::Neerlandes => "Extern display · Verbinding niet geïdentificeerd".to_string(),
            Idioma::NoruegoNynorsk => "Eksternt display · Tilkobling ikke identifisert".to_string(),
            Idioma::Polaco => "Wyświetlacz zewnętrzny · Nie zidentyfikowano połączenia".to_string(),
            Idioma::PortuguesBrasil => "Display externo · Conexão não identificada".to_string(),
            Idioma::Ruso => "Внешний дисплей · Соединение не идентифицировано".to_string(),
            Idioma::Sueco => "Extern display · Anslutning inte identifierad".to_string(),
            Idioma::Turco => "External display · Connection not identified".to_string(),
            Idioma::Ucraniano => "Зовнішній дисплей · Підключення не ідентифіковано".to_string(),
            Idioma::Vietnamita => "Màn hình ngoài · Không xác định được kết nối".to_string(),
            Idioma::ChinoSimplificado => "外部显示器·连接未识别".to_string(),
            Idioma::Hungaro => "Külső kijelző · A csatlakozás nem azonosítható".to_string(),
            Idioma::Espanol => "Pantalla externa · Conexión no identificada".to_string(),
        });
    };

    let conector = pantalla
        .get("connector")
        .and_then(Value::as_str)
        .filter(|valor| matches!(*valor, "HDMI" | "DisplayPort"));

    let nombre = pantalla
        .get("monitorName")
        .and_then(Value::as_str)
        .filter(|valor| !valor.trim().is_empty());

    let nombre = nombre.unwrap_or(match idioma {
        Idioma::Ingles => "External display",
        Idioma::BelarusLatino => "Znješni dyspłjej",
        Idioma::Belarus => "Знешні дысплей",
        Idioma::Catalan => "Pantalla externa",
        Idioma::Checo => "Externí displej",
        Idioma::Aleman => "Externes Display",
        Idioma::Frances => "Écran externe",
        Idioma::Gallego => "Pantalla externa",
        Idioma::Italiano => "Display esterno",
        Idioma::Coreano => "외부 디스플레이",
        Idioma::Kurdo => "Nîşandana derve",
        Idioma::Neerlandes => "Extern beeldscherm",
        Idioma::NoruegoNynorsk => "Ekstern skjerm",
        Idioma::Polaco => "Wyświetlacz zewnętrzny",
        Idioma::PortuguesBrasil => "Tela externa",
        Idioma::Ruso => "Внешний дисплей",
        Idioma::Sueco => "Extern display",
        Idioma::Turco => "Harici ekran",
        Idioma::Ucraniano => "Зовнішній дисплей",
        Idioma::Vietnamita => "Màn hình ngoài",
        Idioma::ChinoSimplificado => "外部显示器",
        Idioma::Hungaro => "Külső kijelző",
        Idioma::Espanol => "Pantalla externa",
    });

    Some(match (idioma, conector) {
        (Idioma::Ingles, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::BelarusLatino, Some(conector)) => format!("{nombre} praz {conector}"),
        (Idioma::Belarus, Some(conector)) => format!("{nombre} праз {conector}"),
        (Idioma::Catalan, Some(conector)) => format!("{nombre} mitjançant {conector}"),
        (Idioma::Checo, Some(conector)) => format!("{nombre} prostřednictvím {conector}"),
        (Idioma::Aleman, Some(conector)) => format!("{nombre} über {conector}"),
        (Idioma::Frances, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::Gallego, Some(conector)) => format!("{nombre} vía {conector}"),
        (Idioma::Italiano, Some(conector)) => format!("{nombre} tramite {conector}"),
        (Idioma::Coreano, Some(conector)) => format!("{nombre}({conector}을 통해)"),
        (Idioma::Kurdo, Some(conector)) => format!("{nombre} bi rêya {conector}"),
        (Idioma::Neerlandes, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::NoruegoNynorsk, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::Polaco, Some(conector)) => format!("{nombre} przez {conector}"),
        (Idioma::PortuguesBrasil, Some(conector)) => format!("{nombre} através de {conector}"),
        (Idioma::Ruso, Some(conector)) => format!("{nombre} через {conector}"),
        (Idioma::Sueco, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::Turco, Some(conector)) => format!("{nombre} via {conector}"),
        (Idioma::Ucraniano, Some(conector)) => format!("{nombre} через {conector}"),
        (Idioma::Vietnamita, Some(conector)) => format!("{nombre} qua {conector}"),
        (Idioma::ChinoSimplificado, Some(conector)) => format!("{nombre} 通过 {conector}"),
        (Idioma::Hungaro, Some(conector)) => format!("{nombre} · {conector}"),
        (Idioma::Espanol, Some(conector)) => format!("{nombre} por {conector}"),
        (Idioma::Ingles, None) => format!("{nombre} · Connection not identified"),
        (Idioma::BelarusLatino, None) => format!("{nombre} · Złučennje nje vyznačana"),
        (Idioma::Belarus, None) => format!("{nombre} · Злучэнне не вызначана"),
        (Idioma::Catalan, None) => format!("{nombre} · Connexió no identificada"),
        (Idioma::Checo, None) => format!("{nombre} · Připojení nebylo identifikováno"),
        (Idioma::Aleman, None) => format!("{nombre} · Verbindung nicht identifiziert"),
        (Idioma::Frances, None) => format!("{nombre} · Connexion non identifiée"),
        (Idioma::Gallego, None) => format!("{nombre} · Conexión non identificada"),
        (Idioma::Italiano, None) => format!("{nombre} · Connessione non identificata"),
        (Idioma::Coreano, None) => format!("{nombre} · 연결이 식별되지 않음"),
        (Idioma::Kurdo, None) => format!("{nombre} · Têkilî nehat naskirin"),
        (Idioma::Neerlandes, None) => format!("{nombre} · Verbinding niet geïdentificeerd"),
        (Idioma::NoruegoNynorsk, None) => format!("{nombre} · Tilkobling ikke identifisert"),
        (Idioma::Polaco, None) => format!("{nombre} · Nie zidentyfikowano połączenia"),
        (Idioma::PortuguesBrasil, None) => format!("{nombre} · Conexão não identificada"),
        (Idioma::Ruso, None) => format!("{nombre} · Соединение не идентифицировано"),
        (Idioma::Sueco, None) => format!("{nombre} · Anslutning inte identifierad"),
        (Idioma::Turco, None) => format!("{nombre} · Connection not identified"),
        (Idioma::Ucraniano, None) => format!("{nombre} · Підключення не ідентифіковано"),
        (Idioma::Vietnamita, None) => format!("{nombre} · Không xác định được kết nối"),
        (Idioma::ChinoSimplificado, None) => format!("{nombre} · 连接未识别"),
        (Idioma::Hungaro, None) => format!("{nombre} · A csatlakozás nem azonosítható"),
        (Idioma::Espanol, None) => format!("{nombre} · Conexión no identificada"),
    })
}

fn modelo_audio_conocido(huella: &str) -> Option<&'static str> {
    let huella = huella.to_ascii_lowercase();

    if huella.contains("hd_pro_webcam_c920")
        || huella.contains("hd pro webcam c920")
        || huella.contains("webcam c920")
        || huella.contains("logitech c920")
    {
        return Some("Logitech C920");
    }

    if huella.contains("webcam c922") || huella.contains("logitech c922") {
        return Some("Logitech C922");
    }

    if huella.contains("logitech brio") || huella.contains("brio webcam") {
        return Some("Logitech Brio");
    }

    if huella.contains("logitech streamcam") || huella.contains("streamcam") {
        return Some("Logitech StreamCam");
    }

    if huella.contains("scarlett 2i2") {
        return Some("Focusrite Scarlett 2i2");
    }

    if huella.contains("scarlett solo") {
        return Some("Focusrite Scarlett Solo");
    }

    if huella.contains("elgato wave:3") || huella.contains("elgato wave 3") {
        return Some("Elgato Wave:3");
    }

    if huella.contains("hyperx quadcast") {
        return Some("HyperX QuadCast");
    }

    if huella.contains("hyperx solocast") {
        return Some("HyperX SoloCast");
    }

    None
}

fn descripcion_nodo_audio(
    idioma: Idioma,
    audio: &Value,
    clase: &str,
    id: u64,
    respaldo: &str,
) -> String {
    let puntero = if clase == "sink" {
        "/pulse/sinks"
    } else {
        "/pulse/sources"
    };

    let item = audio
        .pointer(puntero)
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("index").and_then(Value::as_u64) == Some(id))
        });

    if clase == "sink" {
        if let Some(pantalla) = descripcion_pantalla_audio(idioma, audio, item, respaldo) {
            return pantalla;
        }
    }

    let descripcion = item
        .and_then(|item| item.get("description"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|valor| !valor.is_empty() && *valor != "(null)")
        .unwrap_or("");

    let puerto = item
        .and_then(|item| item.get("activePort"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let huella = format!("{respaldo} {descripcion}").to_ascii_lowercase();

    if clase == "source" {
        if let Some(modelo) = modelo_audio_conocido(&huella) {
            return modelo.to_string();
        }

        return match puerto {
            "analog-input-front-mic" => match idioma {
                Idioma::Ingles => "Front microphone".to_string(),
                Idioma::BelarusLatino => "Pjaredni mikrafon".to_string(),
                Idioma::Belarus => "Пярэдні мікрафон".to_string(),
                Idioma::Catalan => "Micròfon frontal".to_string(),
                Idioma::Checo => "Přední mikrofon".to_string(),
                Idioma::Aleman => "Frontmikrofon".to_string(),
                Idioma::Frances => "Microphone avant".to_string(),
                Idioma::Gallego => "Micrófono frontal".to_string(),
                Idioma::Italiano => "Microfono frontale".to_string(),
                Idioma::Coreano => "전면 마이크".to_string(),
                Idioma::Kurdo => "Mîkrofona pêşîn".to_string(),
                Idioma::Neerlandes => "Microfoon aan de voorkant".to_string(),
                Idioma::NoruegoNynorsk => "Mikrofon foran".to_string(),
                Idioma::Polaco => "Mikrofon przedni".to_string(),
                Idioma::PortuguesBrasil => "Microfone frontal".to_string(),
                Idioma::Ruso => "Передний микрофон".to_string(),
                Idioma::Sueco => "Främre mikrofon".to_string(),
                Idioma::Turco => "Ön mikrofon".to_string(),
                Idioma::Ucraniano => "Передній мікрофон".to_string(),
                Idioma::Vietnamita => "Micrô phía trước".to_string(),
                Idioma::ChinoSimplificado => "前置麦克风".to_string(),
                Idioma::Hungaro => "Elülső mikrofon".to_string(),
                Idioma::Espanol => "Micrófono delantero".to_string(),
            },
            "analog-input-rear-mic" => match idioma {
                Idioma::Ingles => "Rear microphone".to_string(),
                Idioma::BelarusLatino => "Zadni mikrafon".to_string(),
                Idioma::Belarus => "Задні мікрафон".to_string(),
                Idioma::Catalan => "Micròfon posterior".to_string(),
                Idioma::Checo => "Zadní mikrofon".to_string(),
                Idioma::Aleman => "Hinteres Mikrofon".to_string(),
                Idioma::Frances => "Microphone arrière".to_string(),
                Idioma::Gallego => "Micrófono traseiro".to_string(),
                Idioma::Italiano => "Microfono posteriore".to_string(),
                Idioma::Coreano => "후면 마이크".to_string(),
                Idioma::Kurdo => "Mîkrofona paşîn".to_string(),
                Idioma::Neerlandes => "Achtermicrofoon".to_string(),
                Idioma::NoruegoNynorsk => "Mikrofon bak".to_string(),
                Idioma::Polaco => "Mikrofon tylny".to_string(),
                Idioma::PortuguesBrasil => "Microfone traseiro".to_string(),
                Idioma::Ruso => "Задний микрофон".to_string(),
                Idioma::Sueco => "Bakre mikrofon".to_string(),
                Idioma::Turco => "Arka mikrofon".to_string(),
                Idioma::Ucraniano => "Задній мікрофон".to_string(),
                Idioma::Vietnamita => "Micrô phía sau".to_string(),
                Idioma::ChinoSimplificado => "后麦克风".to_string(),
                Idioma::Hungaro => "Hátsó mikrofon".to_string(),
                Idioma::Espanol => "Micrófono trasero".to_string(),
            },
            "analog-input-mic" => match idioma {
                Idioma::Ingles => "Microphone".to_string(),
                Idioma::BelarusLatino => "Mikrafon".to_string(),
                Idioma::Belarus => "Мікрафон".to_string(),
                Idioma::Catalan => "Micròfon".to_string(),
                Idioma::Checo => "Mikrofon".to_string(),
                Idioma::Aleman => "Mikrofon".to_string(),
                Idioma::Frances => "Micro".to_string(),
                Idioma::Gallego => "Micrófono".to_string(),
                Idioma::Italiano => "Microfono".to_string(),
                Idioma::Coreano => "마이크".to_string(),
                Idioma::Kurdo => "Mîkrofon".to_string(),
                Idioma::Neerlandes => "Microfoon".to_string(),
                Idioma::NoruegoNynorsk => "Mikrofon".to_string(),
                Idioma::Polaco => "Mikrofon".to_string(),
                Idioma::PortuguesBrasil => "Microfone".to_string(),
                Idioma::Ruso => "Микрофон".to_string(),
                Idioma::Sueco => "Mikrofon".to_string(),
                Idioma::Turco => "Mikrofon".to_string(),
                Idioma::Ucraniano => "Мікрофон".to_string(),
                Idioma::Vietnamita => "Micrô".to_string(),
                Idioma::ChinoSimplificado => "麦克风".to_string(),
                Idioma::Hungaro => "Mikrofon".to_string(),
                Idioma::Espanol => "Micrófono".to_string(),
            },
            "analog-input-linein" => match idioma {
                Idioma::Ingles => "Line input".to_string(),
                Idioma::BelarusLatino => "Łinjejny ŭvod".to_string(),
                Idioma::Belarus => "Радковы ўвод".to_string(),
                Idioma::Catalan => "Entrada de línia".to_string(),
                Idioma::Checo => "Linkový vstup".to_string(),
                Idioma::Aleman => "Leitungseingang".to_string(),
                Idioma::Frances => "Entrée ligne".to_string(),
                Idioma::Gallego => "Entrada de liña".to_string(),
                Idioma::Italiano => "Ingresso linea".to_string(),
                Idioma::Coreano => "라인 입력".to_string(),
                Idioma::Kurdo => "Têketina xetê".to_string(),
                Idioma::Neerlandes => "Lijningang".to_string(),
                Idioma::NoruegoNynorsk => "Linjeinngang".to_string(),
                Idioma::Polaco => "Wejście liniowe".to_string(),
                Idioma::PortuguesBrasil => "Entrada de linha".to_string(),
                Idioma::Ruso => "Линейный вход".to_string(),
                Idioma::Sueco => "Linjeingång".to_string(),
                Idioma::Turco => "Hat girişi".to_string(),
                Idioma::Ucraniano => "Рядковий вхід".to_string(),
                Idioma::Vietnamita => "Đầu vào dòng".to_string(),
                Idioma::ChinoSimplificado => "线路输入".to_string(),
                Idioma::Hungaro => "Vonalbemenet".to_string(),
                Idioma::Espanol => "Entrada de línea".to_string(),
            },
            _ if !descripcion.is_empty() && !parece_nombre_tecnico_audio(descripcion) => {
                descripcion.to_string()
            }
            _ => match idioma {
                Idioma::Ingles => "Microphone".to_string(),
                Idioma::BelarusLatino => "Mikrafon".to_string(),
                Idioma::Belarus => "Мікрафон".to_string(),
                Idioma::Catalan => "Micròfon".to_string(),
                Idioma::Checo => "Mikrofon".to_string(),
                Idioma::Aleman => "Mikrofon".to_string(),
                Idioma::Frances => "Micro".to_string(),
                Idioma::Gallego => "Micrófono".to_string(),
                Idioma::Italiano => "Microfono".to_string(),
                Idioma::Coreano => "마이크".to_string(),
                Idioma::Kurdo => "Mîkrofon".to_string(),
                Idioma::Neerlandes => "Microfoon".to_string(),
                Idioma::NoruegoNynorsk => "Mikrofon".to_string(),
                Idioma::Polaco => "Mikrofon".to_string(),
                Idioma::PortuguesBrasil => "Microfone".to_string(),
                Idioma::Ruso => "Микрофон".to_string(),
                Idioma::Sueco => "Mikrofon".to_string(),
                Idioma::Turco => "Mikrofon".to_string(),
                Idioma::Ucraniano => "Мікрофон".to_string(),
                Idioma::Vietnamita => "Micrô".to_string(),
                Idioma::ChinoSimplificado => "麦克风".to_string(),
                Idioma::Hungaro => "Mikrofon".to_string(),
                Idioma::Espanol => "Micrófono".to_string(),
            },
        };
    }

    match puerto {
        "analog-output-headphones" => match idioma {
            Idioma::Ingles => "Headphones".to_string(),
            Idioma::BelarusLatino => "Navušniki".to_string(),
            Idioma::Belarus => "Навушнікі".to_string(),
            Idioma::Catalan => "Auriculars".to_string(),
            Idioma::Checo => "Sluchátka".to_string(),
            Idioma::Aleman => "Kopfhörer".to_string(),
            Idioma::Frances => "Écouteurs".to_string(),
            Idioma::Gallego => "Auriculares".to_string(),
            Idioma::Italiano => "Cuffie".to_string(),
            Idioma::Coreano => "헤드폰".to_string(),
            Idioma::Kurdo => "Headphones".to_string(),
            Idioma::Neerlandes => "Koptelefoon".to_string(),
            Idioma::NoruegoNynorsk => "Hodetelefoner".to_string(),
            Idioma::Polaco => "Słuchawki".to_string(),
            Idioma::PortuguesBrasil => "Fones de ouvido".to_string(),
            Idioma::Ruso => "Наушники".to_string(),
            Idioma::Sueco => "Hörlurar".to_string(),
            Idioma::Turco => "Kulaklık".to_string(),
            Idioma::Ucraniano => "Навушники".to_string(),
            Idioma::Vietnamita => "Tai Nghe".to_string(),
            Idioma::ChinoSimplificado => "耳机".to_string(),
            Idioma::Hungaro => "Fejhallgató".to_string(),
            Idioma::Espanol => "Auriculares".to_string(),
        },
        "analog-output-lineout" => match idioma {
            Idioma::Ingles => "Line out".to_string(),
            Idioma::BelarusLatino => "Łinija z".to_string(),
            Idioma::Belarus => "Лінейны выхад".to_string(),
            Idioma::Catalan => "Sortida de línia".to_string(),
            Idioma::Checo => "Linka ven".to_string(),
            Idioma::Aleman => "Line-Out".to_string(),
            Idioma::Frances => "Sortie ligne".to_string(),
            Idioma::Gallego => "Saída de liña".to_string(),
            Idioma::Italiano => "Linea fuori".to_string(),
            Idioma::Coreano => "라인 출력".to_string(),
            Idioma::Kurdo => "Line der".to_string(),
            Idioma::Neerlandes => "Lijnuitgang".to_string(),
            Idioma::NoruegoNynorsk => "Linje ut".to_string(),
            Idioma::Polaco => "Wyjście liniowe".to_string(),
            Idioma::PortuguesBrasil => "Saída de linha".to_string(),
            Idioma::Ruso => "Линейный выход".to_string(),
            Idioma::Sueco => "Linje ut".to_string(),
            Idioma::Turco => "Hat çıkışı".to_string(),
            Idioma::Ucraniano => "Лінійний вихід".to_string(),
            Idioma::Vietnamita => "Đầu ra".to_string(),
            Idioma::ChinoSimplificado => "线路输出".to_string(),
            Idioma::Hungaro => "Vonalkimenet".to_string(),
            Idioma::Espanol => "Salida de línea".to_string(),
        },
        "analog-output-speaker" | "analog-output-speakers" => match idioma {
            Idioma::Ingles => "Speakers".to_string(),
            Idioma::BelarusLatino => "Vystupoŭcy".to_string(),
            Idioma::Belarus => "Дынамікі".to_string(),
            Idioma::Catalan => "Altaveus".to_string(),
            Idioma::Checo => "Reproduktory".to_string(),
            Idioma::Aleman => "Lautsprecher".to_string(),
            Idioma::Frances => "Haut-parleurs".to_string(),
            Idioma::Gallego => "Altofalantes".to_string(),
            Idioma::Italiano => "Altoparlanti".to_string(),
            Idioma::Coreano => "스피커".to_string(),
            Idioma::Kurdo => "Axaftvan".to_string(),
            Idioma::Neerlandes => "Luidsprekers".to_string(),
            Idioma::NoruegoNynorsk => "Høyttalere".to_string(),
            Idioma::Polaco => "Głośniki".to_string(),
            Idioma::PortuguesBrasil => "Alto-falantes".to_string(),
            Idioma::Ruso => "Динамики".to_string(),
            Idioma::Sueco => "Högtalare".to_string(),
            Idioma::Turco => "Hoparlörler".to_string(),
            Idioma::Ucraniano => "Динаміки".to_string(),
            Idioma::Vietnamita => "Loa".to_string(),
            Idioma::ChinoSimplificado => "扬声器".to_string(),
            Idioma::Hungaro => "Hangszórók".to_string(),
            Idioma::Espanol => "Altavoces".to_string(),
        },
        _ if !descripcion.is_empty() && !parece_nombre_tecnico_audio(descripcion) => {
            descripcion.to_string()
        }
        _ => match idioma {
            Idioma::Ingles => "Sound output".to_string(),
            Idioma::BelarusLatino => "Hukavy vychad".to_string(),
            Idioma::Belarus => "Гукавы выхад".to_string(),
            Idioma::Catalan => "Sortida de so".to_string(),
            Idioma::Checo => "Zvukový výstup".to_string(),
            Idioma::Aleman => "Tonausgabe".to_string(),
            Idioma::Frances => "Sortie sonore".to_string(),
            Idioma::Gallego => "Saída de son".to_string(),
            Idioma::Italiano => "Uscita audio".to_string(),
            Idioma::Coreano => "사운드 출력".to_string(),
            Idioma::Kurdo => "Derketina deng".to_string(),
            Idioma::Neerlandes => "Geluidsuitvoer".to_string(),
            Idioma::NoruegoNynorsk => "Lydutgang".to_string(),
            Idioma::Polaco => "Wyjście dźwięku".to_string(),
            Idioma::PortuguesBrasil => "Saída de som".to_string(),
            Idioma::Ruso => "Звуковой выход".to_string(),
            Idioma::Sueco => "Ljudutgång".to_string(),
            Idioma::Turco => "Ses çıkışı".to_string(),
            Idioma::Ucraniano => "Звуковий вихід".to_string(),
            Idioma::Vietnamita => "Âm thanh phát ra".to_string(),
            Idioma::ChinoSimplificado => "声音输出".to_string(),
            Idioma::Hungaro => "Hangkimenet".to_string(),
            Idioma::Espanol => "Salida de sonido".to_string(),
        },
    }
}

fn fila_navegacion(titulo: &str, icono: &str) -> gtk::ListBoxRow {
    let fila = gtk::ListBoxRow::new();
    fila.set_activatable(true);
    fila.set_selectable(true);
    fila.set_size_request(-1, 60);

    let contenido = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    contenido.set_margin_top(8);
    contenido.set_margin_bottom(8);
    contenido.set_margin_start(10);
    contenido.set_margin_end(10);

    let imagen = gtk::Image::from_icon_name(icono);
    imagen.set_pixel_size(18);

    let titulo = localizar_visible(idioma_actual(), titulo);
    let etiqueta = gtk::Label::new(Some(&titulo));
    etiqueta.set_xalign(0.0);
    etiqueta.set_hexpand(true);
    etiqueta.set_ellipsize(gtk::pango::EllipsizeMode::End);

    contenido.append(&imagen);
    contenido.append(&etiqueta);
    fila.set_child(Some(&contenido));
    fila
}

fn fila(titulo: &str, contenido: impl AsRef<str>) -> adw::ActionRow {
    let fila = adw::ActionRow::new();
    let idioma = idioma_actual();
    let titulo = localizar_visible(idioma, titulo);
    let contenido = localizar_visible(idioma, contenido.as_ref());
    fila.set_title(&titulo);
    fila.set_subtitle(&contenido);
    fila
}

fn texto_error_amigable(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::Ingles => {
            "The operation did not complete. Korunix does not consider the change applied. You can close this message, refresh the area and try again. Technical details are optional."
        }
        Idioma::BelarusLatino => {
            "Apjeracyja nje zavjeršana. Korunix nje łičyc zmjeny prymjenjenymi. Vy možacje zakryc heta pavjedamłjennje, abnavic vobłasc i paŭtaryc sprobu. Techničnyja detałi njeabavjazkovyja."
        }
        Idioma::Belarus => {
            "Аперацыя не завершана. Korunix не лічыць змены прымененымі. Вы можаце закрыць гэта паведамленне, абнавіць вобласць і паўтарыць спробу. Тэхнічныя дэталі неабавязковыя."
        }
        Idioma::Catalan => {
            "L'operació no s'ha completat. Korunix no considera aplicat el canvi. Pots tancar aquest missatge, actualitzar l'àrea i tornar-ho a provar. Els detalls tècnics són opcionals."
        }
        Idioma::Checo => {
            "Operace nebyla dokončena. Korunix změnu nepovažuje za aplikovanou. Tuto zprávu můžete zavřít, obnovit oblast a zkusit to znovu. Technické detaily jsou volitelné."
        }
        Idioma::Aleman => {
            "Der Vorgang wurde nicht abgeschlossen. Korunix betrachtet die Änderung nicht als angewendet. Sie können diese Meldung schließen, den Bereich aktualisieren und es erneut versuchen. Technische Details sind optional."
        }
        Idioma::Frances => {
            "L'opération n'a pas abouti. Korunix ne considère pas le changement appliqué. Vous pouvez fermer ce message, actualiser la zone et réessayer. Les détails techniques sont facultatifs."
        }
        Idioma::Gallego => {
            "A operación non se completou. Korunix non considera o cambio aplicado. Podes pechar esta mensaxe, actualizar a área e tentalo de novo. Os detalles técnicos son opcionais."
        }
        Idioma::Italiano => {
            "L'operazione non è stata completata. Korunix non considera la modifica applicata. Puoi chiudere questo messaggio, aggiornare l'area e riprovare. I dettagli tecnici sono facoltativi."
        }
        Idioma::Coreano => {
            "작업이 완료되지 않았습니다. Korunix는 적용된 변경 사항을 고려하지 않습니다. 이 메시지를 닫고 해당 영역을 새로 고친 후 다시 시도해 보세요. 기술적인 세부사항은 선택사항입니다."
        }
        Idioma::Kurdo => {
            "Operasyon bi dawî nebû. Korunix guhertina ku hatiye kirin nahesibîne. Hûn dikarin vê peyamê bigirin, deverê nûve bikin û dîsa biceribînin. Agahiyên teknîkî vebijarkî ne."
        }
        Idioma::Neerlandes => {
            "De bewerking is niet voltooid. Korunix beschouwt de wijziging niet als toegepast. U kunt dit bericht sluiten, het gebied vernieuwen en het opnieuw proberen. Technische details zijn optioneel."
        }
        Idioma::NoruegoNynorsk => {
            "Operasjonen ble ikke fullført. Korunix anser ikke endringen som anvendt. Du kan lukke denne meldingen, oppdatere området og prøve på nytt. Tekniske detaljer er valgfrie."
        }
        Idioma::Polaco => {
            "Operacja nie została ukończona. Korunix nie uznaje zastosowanej zmiany. Możesz zamknąć tę wiadomość, odświeżyć obszar i spróbować ponownie. Szczegóły techniczne są opcjonalne."
        }
        Idioma::PortuguesBrasil => {
            "A operação não foi concluída. Korunix não considera a alteração aplicada. Você pode fechar esta mensagem, atualizar a área e tentar novamente. Os detalhes técnicos são opcionais."
        }
        Idioma::Ruso => {
            "Операция не завершена. Korunix не учитывает внесенные изменения. Вы можете закрыть это сообщение, обновить область и повторить попытку. Технические подробности не являются обязательными."
        }
        Idioma::Sueco => {
            "Åtgärden slutfördes inte. Korunix anser inte ändringen tillämpad. Du kan stänga det här meddelandet, uppdatera området och försöka igen. Tekniska detaljer är valfria."
        }
        Idioma::Turco => {
            "İşlem tamamlanmadı. Korunix uygulanan değişikliği dikkate almaz. You can close this message, refresh the area and try again. Teknik detaylar isteğe bağlıdır."
        }
        Idioma::Ucraniano => {
            "Операцію не завершено. Korunix не вважає зміну застосованою. Ви можете закрити це повідомлення, оновити область і повторити спробу. Технічні дані необов'язкові."
        }
        Idioma::Vietnamita => {
            "Thao tác chưa hoàn tất. Korunix không xem xét thay đổi được áp dụng. Bạn có thể đóng thông báo này, làm mới khu vực và thử lại. Chi tiết kỹ thuật là tùy chọn."
        }
        Idioma::ChinoSimplificado => {
            "操作未完成。 Korunix 不考虑已应用的更改。您可以关闭此消息，刷新该区域，然后重试。技术细节是可选的。"
        }
        Idioma::Hungaro => {
            "A művelet nem fejeződött be. A Korunix nem tekinti alkalmazottnak a módosítást. Bezárhatod ezt az üzenetet, frissítheted a területet és újra próbálhatod. A műszaki részletek opcionálisak."
        }
        Idioma::Espanol => {
            "La operación no se completó. Korunix no considera aplicado el cambio. Puedes cerrar este aviso, actualizar el área e intentarlo de nuevo. Los detalles técnicos son opcionales."
        }
    }
}

fn texto_detalles_tecnicos(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::Ingles => "Technical details",
        Idioma::BelarusLatino => "Techničnyja detałi",
        Idioma::Belarus => "Тэхнічныя дэталі",
        Idioma::Catalan => "Detalls tècnics",
        Idioma::Checo => "Technické detaily",
        Idioma::Aleman => "Technische Details",
        Idioma::Frances => "Détails techniques",
        Idioma::Gallego => "Datos técnicos",
        Idioma::Italiano => "Dettagli tecnici",
        Idioma::Coreano => "기술 세부사항",
        Idioma::Kurdo => "Hûrguliyên teknîkî",
        Idioma::Neerlandes => "Technische details",
        Idioma::NoruegoNynorsk => "Tekniske detaljer",
        Idioma::Polaco => "Szczegóły techniczne",
        Idioma::PortuguesBrasil => "Detalhes técnicos",
        Idioma::Ruso => "Технические подробности",
        Idioma::Sueco => "Tekniska detaljer",
        Idioma::Turco => "Teknik detaylar",
        Idioma::Ucraniano => "Технічні деталі",
        Idioma::Vietnamita => "Chi tiết kỹ thuật",
        Idioma::ChinoSimplificado => "技术细节",
        Idioma::Hungaro => "Műszaki részletek",
        Idioma::Espanol => "Detalles técnicos",
    }
}

fn texto_mostrar(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::Ingles => "Show",
        Idioma::BelarusLatino => "Pakazac",
        Idioma::Belarus => "Паказаць",
        Idioma::Catalan => "Mostra",
        Idioma::Checo => "Zobrazit",
        Idioma::Aleman => "Anzeigen",
        Idioma::Frances => "Afficher",
        Idioma::Gallego => "Mostrar",
        Idioma::Italiano => "Mostra",
        Idioma::Coreano => "표시",
        Idioma::Kurdo => "Nîşan bide",
        Idioma::Neerlandes => "Toon",
        Idioma::NoruegoNynorsk => "Vis",
        Idioma::Polaco => "Pokaż",
        Idioma::PortuguesBrasil => "Exibir",
        Idioma::Ruso => "Показать",
        Idioma::Sueco => "Visa",
        Idioma::Turco => "Göster",
        Idioma::Ucraniano => "Показати",
        Idioma::Vietnamita => "Hiển thị",
        Idioma::ChinoSimplificado => "显示",
        Idioma::Hungaro => "Mutatás",
        Idioma::Espanol => "Mostrar",
    }
}

fn texto_ocultar(idioma: Idioma) -> &'static str {
    match idioma {
        Idioma::Ingles => "Hide",
        Idioma::BelarusLatino => "Schavac",
        Idioma::Belarus => "Схаваць",
        Idioma::Catalan => "Amaga",
        Idioma::Checo => "Skrýt",
        Idioma::Aleman => "Ausblenden",
        Idioma::Frances => "Cacher",
        Idioma::Gallego => "Ocultar",
        Idioma::Italiano => "Nascondi",
        Idioma::Coreano => "숨기기",
        Idioma::Kurdo => "Veşêre",
        Idioma::Neerlandes => "Verberg",
        Idioma::NoruegoNynorsk => "Gøym",
        Idioma::Polaco => "Ukryj",
        Idioma::PortuguesBrasil => "Ocultar",
        Idioma::Ruso => "Скрыть",
        Idioma::Sueco => "Dölj",
        Idioma::Turco => "Gizle",
        Idioma::Ucraniano => "Приховати",
        Idioma::Vietnamita => "Ẩn",
        Idioma::ChinoSimplificado => "隐藏",
        Idioma::Hungaro => "Elrejtés",
        Idioma::Espanol => "Ocultar",
    }
}

fn pagina_error(idioma: Idioma, detalle: &str) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();

    grupo.set_title(texto(idioma, "error"));
    grupo.add(&fila(texto(idioma, "status"), texto_error_amigable(idioma)));

    let control = adw::ActionRow::new();
    control.set_title(texto_detalles_tecnicos(idioma));

    let boton = gtk::Button::with_label(texto_mostrar(idioma));
    boton.set_valign(gtk::Align::Center);
    control.add_suffix(&boton);
    grupo.add(&control);

    let tecnico = fila(texto_detalles_tecnicos(idioma), detalle);
    tecnico.set_visible(false);
    grupo.add(&tecnico);

    let tecnico_boton = tecnico.clone();
    boton.connect_clicked(move |boton| {
        let visible = !tecnico_boton.is_visible();
        tecnico_boton.set_visible(visible);
        boton.set_label(if visible {
            texto_ocultar(idioma)
        } else {
            texto_mostrar(idioma)
        });
    });

    pagina.add(&grupo);
    pagina
}

fn ahora_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn tiempo_relativo_desde(timestamp: u64, ahora: u64) -> String {
    let diferencia = ahora.saturating_sub(timestamp);

    if diferencia < 60 {
        "Ahora".to_string()
    } else if diferencia < 3600 {
        format!("Hace {} min", diferencia / 60)
    } else if diferencia < 86_400 {
        format!("Hace {} h", diferencia / 3600)
    } else {
        let dias = diferencia / 86_400;
        if dias == 1 {
            "Hace 1 día".to_string()
        } else {
            format!("Hace {dias} días")
        }
    }
}

fn tiempo_relativo(timestamp: u64) -> String {
    tiempo_relativo_desde(timestamp, ahora_epoch())
}

fn ultima_copia_portable(historial: &Value) -> Option<u64> {
    historial
        .get("entries")
        .and_then(Value::as_array)?
        .iter()
        .rev()
        .find(|entrada| entrada.get("kind").and_then(Value::as_str) == Some("backup-export"))
        .and_then(|entrada| entrada.get("timestamp"))
        .and_then(Value::as_u64)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AsuntoResumen {
    titulo: String,
    detalle: String,
    destino: &'static str,
}

fn asuntos_resumen(
    historial: Option<&Value>,
    firmware: Option<&Value>,
    privilegios: Option<&Value>,
    ahora: u64,
) -> Vec<AsuntoResumen> {
    let mut asuntos = Vec::new();

    match historial {
        Some(historial) => match ultima_copia_portable(historial) {
            Some(timestamp) => {
                let dias = ahora.saturating_sub(timestamp) / 86_400;
                if dias >= 30 {
                    asuntos.push(AsuntoResumen {
                        titulo: "Copia de seguridad".to_string(),
                        detalle: format!(
                            "La última copia portable tiene {dias} días. Conviene crear una nueva."
                        ),
                        destino: "backups",
                    });
                }
            }
            None => asuntos.push(AsuntoResumen {
                titulo: "Copia de seguridad".to_string(),
                detalle: "Todavía no has creado una copia portable de esta configuración."
                    .to_string(),
                destino: "backups",
            }),
        },
        None => asuntos.push(AsuntoResumen {
            titulo: "Copias e historial".to_string(),
            detalle: "Korunix no pudo comprobar el estado de las copias.".to_string(),
            destino: "backups",
        }),
    }

    match firmware {
        Some(firmware) => {
            let disponibles = cantidad(firmware, "/devices");
            if disponibles > 0 {
                asuntos.push(AsuntoResumen {
                    titulo: "Firmware".to_string(),
                    detalle: if disponibles == 1 {
                        "Hay una actualización de firmware disponible.".to_string()
                    } else {
                        format!("Hay {disponibles} actualizaciones de firmware disponibles.")
                    },
                    destino: "firmware",
                });
            }
        }
        None => asuntos.push(AsuntoResumen {
            titulo: "Firmware".to_string(),
            detalle: "Korunix no pudo comprobar el estado del firmware.".to_string(),
            destino: "firmware",
        }),
    }

    match privilegios {
        Some(privilegios) => {
            if privilegios.get("guiUsable").and_then(Value::as_bool) == Some(false) {
                asuntos.push(AsuntoResumen {
                    titulo: "Permisos".to_string(),
                    detalle:
                        "Korunix no puede solicitar los permisos necesarios para modificar el sistema."
                            .to_string(),
                    destino: "maintenance",
                });
            }
        }
        None => asuntos.push(AsuntoResumen {
            titulo: "Permisos".to_string(),
            detalle: "Korunix no pudo comprobar si podrá autorizar cambios del sistema."
                .to_string(),
            destino: "maintenance",
        }),
    }

    asuntos
}

fn indice_pagina(nombre: &str) -> Option<i32> {
    match nombre {
        "summary" => Some(0),
        "updates" => Some(1),
        "hardware" => Some(2),
        "media" => Some(3),
        "storage" => Some(4),
        "firmware" => Some(5),
        "applications" => Some(6),
        "appearance" => Some(7),
        "localization" => Some(8),
        "people" => Some(9),
        "backups" => Some(10),
        "maintenance" => Some(11),
        _ => None,
    }
}

fn clave_titulo_pagina(nombre: &str) -> &'static str {
    match nombre {
        "updates" => "updates",
        "hardware" => "hardware",
        "media" => "media",
        "storage" => "storage",
        "firmware" => "firmware_updates",
        "applications" => "applications",
        "appearance" => "appearance_desktops",
        "localization" => "localization",
        "people" => "people",
        "backups" => "backups_history",
        "maintenance" => "maintenance",
        _ => "summary",
    }
}

fn navegar_a(estado: &Estado, nombre: &'static str) {
    estado.stack.set_visible_child_name(nombre);
    estado
        .pagina_contenido
        .set_title(texto(estado.idioma, clave_titulo_pagina(nombre)));

    if let Some(indice) = indice_pagina(nombre) {
        if let Some(fila) = estado.navegacion.row_at_index(indice) {
            estado.navegacion.select_row(Some(&fila));
        }
    }
}

fn pagina_resumen(
    estado: Rc<Estado>,
    hardware: &Value,
    people: &Value,
    channel: &Value,
    historial: Option<&Value>,
    firmware: Option<&Value>,
    privilegios: Option<&Value>,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let asuntos = asuntos_resumen(historial, firmware, privilegios, ahora_epoch());

    let grupo_estado = adw::PreferencesGroup::new();
    grupo_estado.set_title(&localizar_visible(idioma_actual(), "Estado del equipo"));

    let estado_general = adw::ActionRow::new();
    if asuntos.is_empty() {
        estado_general.set_title(&localizar_visible(idioma_actual(), "Todo está bien"));
        estado_general.set_subtitle(&localizar_visible(
            idioma_actual(),
            "Korunix no detectó asuntos que requieran atención en las comprobaciones disponibles.",
        ));
    } else {
        estado_general.set_title(&localizar_visible(
            idioma_actual(),
            if asuntos.len() == 1 {
                "Hay un asunto que revisar"
            } else {
                "Hay asuntos que revisar"
            },
        ));
        estado_general.set_subtitle(&localizar_visible(
            idioma_actual(),
            &format!(
                "{} {} requieren tu atención.",
                asuntos.len(),
                if asuntos.len() == 1 {
                    "área"
                } else {
                    "áreas"
                }
            ),
        ));
    }
    grupo_estado.add(&estado_general);
    pagina.add(&grupo_estado);

    if !asuntos.is_empty() {
        let grupo_asuntos = adw::PreferencesGroup::new();
        grupo_asuntos.set_title(&localizar_visible(idioma_actual(), "Necesita atención"));

        for asunto in asuntos {
            let row = adw::ActionRow::new();
            row.set_title(&localizar_visible(idioma_actual(), &asunto.titulo));
            row.set_subtitle(&localizar_visible(idioma_actual(), &asunto.detalle));

            let boton = gtk::Button::with_label(&localizar_visible(idioma_actual(), "Revisar"));
            boton.set_valign(gtk::Align::Center);
            row.add_suffix(&boton);
            grupo_asuntos.add(&row);

            let estado_navegar = Rc::clone(&estado);
            boton.connect_clicked(move |_| {
                navegar_a(&estado_navegar, asunto.destino);
            });
        }

        pagina.add(&grupo_asuntos);
    }

    let grupo_contexto = adw::PreferencesGroup::new();
    grupo_contexto.set_title(&localizar_visible(idioma_actual(), "Este equipo"));

    let vendor = valor(hardware, "/machine/vendor");
    let model = valor(hardware, "/machine/model");
    grupo_contexto.add(&fila(
        texto(estado.idioma, "model"),
        modelo_humano(&vendor, &model),
    ));

    grupo_contexto.add(&fila(
        &localizar_visible(idioma_actual(), "Canal del sistema"),
        valor(channel, "/label"),
    ));

    let personas = people
        .pointer("/accounts")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    grupo_contexto.add(&fila(texto(estado.idioma, "people"), personas.to_string()));

    pagina.add(&grupo_contexto);
    pagina
}

fn pagina_hardware(estado: &Estado, hardware: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    let vendor = valor(hardware, "/machine/vendor");
    let model = valor(hardware, "/machine/model");
    let modelo = modelo_humano(&vendor, &model);

    grupo.add(&fila(texto(estado.idioma, "model"), modelo));
    grupo.add(&fila(
        texto(estado.idioma, "cpu"),
        valor(hardware, "/cpu/model"),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "memory"),
        memoria_humana(hardware),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "boot"),
        firmware_humano(&valor(hardware, "/firmware/detected")),
    ));

    pagina.add(&grupo);
    pagina
}

fn pagina_localizacion(estado: Rc<Estado>, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(&localizar_visible(
        idioma_actual(),
        "Idioma, región y teclado",
    ));

    let actual_idioma = valor(datos, "/declared/systemLanguage");
    let actual_region = valor(datos, "/declared/region");
    let actual_zona = valor(datos, "/declared/timeZone");
    let actual_teclado = datos
        .pointer("/declared/keyboard/layout")
        .and_then(Value::as_str)
        .or_else(|| {
            datos
                .pointer("/derived/keyboard/layout")
                .and_then(Value::as_str)
        })
        .unwrap_or("es")
        .to_string();

    let resumen = format!(
        "{} · {} · {} · {}",
        idioma_humano(estado.idioma, &actual_idioma),
        region_humana(estado.idioma, &actual_region),
        zona_horaria_humana(estado.idioma, &actual_zona),
        teclado_humano(estado.idioma, &actual_teclado),
    );

    let grupo_actual = adw::PreferencesGroup::new();
    grupo_actual.set_title(&localizar_visible(idioma_actual(), "Configuración actual"));
    grupo_actual.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Idioma, región, hora y teclado son decisiones independientes.",
    )));
    grupo_actual.add(&fila(
        "Idioma del sistema",
        idioma_humano(estado.idioma, &actual_idioma),
    ));
    grupo_actual.add(&fila(
        "País o región",
        region_humana(estado.idioma, &actual_region),
    ));
    grupo_actual.add(&fila(
        "Hora local",
        zona_horaria_humana(estado.idioma, &actual_zona),
    ));

    let variante_actual = datos
        .pointer("/declared/keyboard/variant")
        .and_then(Value::as_str)
        .unwrap_or("");
    let teclado_visible = format!(
        "{} · {}",
        teclado_humano(estado.idioma, &actual_teclado),
        variante_teclado_humana(variante_actual),
    );
    grupo_actual.add(&fila("Teclado", teclado_visible));

    let personalizar_fila = adw::ActionRow::new();
    personalizar_fila.set_title(&localizar_visible(idioma_actual(), "Personalizar"));
    personalizar_fila.set_subtitle(&localizar_visible(
        idioma_actual(),
        "Cambia idioma, formatos, zona horaria o teclado por separado.",
    ));
    let personalizar = gtk::Button::with_label(&localizar_visible(idioma_actual(), "Personalizar"));
    personalizar.set_valign(gtk::Align::Center);
    personalizar_fila.add_suffix(&personalizar);
    grupo_actual.add(&personalizar_fila);
    pagina.add(&grupo_actual);

    grupo.set_title(&localizar_visible(idioma_actual(), "Personalizar"));
    grupo.set_description(Some(&format!(
        "{resumen}. Los identificadores técnicos solo se muestran aquí porque esta edición avanzada todavía trabaja directamente con los valores del sistema."
    )));
    grupo.set_visible(false);

    let idioma = adw::EntryRow::new();
    idioma.set_title(&localizar_visible(idioma_actual(), "Idioma del sistema"));
    idioma.set_text(&valor(datos, "/declared/systemLanguage"));
    grupo.add(&idioma);

    let region = adw::EntryRow::new();
    region.set_title(&localizar_visible(idioma_actual(), "Región"));
    region.set_text(&valor(datos, "/declared/region"));
    grupo.add(&region);

    let formatos_idioma = adw::EntryRow::new();
    formatos_idioma.set_title(&localizar_visible(idioma_actual(), "Idioma de formatos"));
    let formato_idioma_actual = datos
        .pointer("/declared/formats/language")
        .and_then(Value::as_str)
        .or_else(|| {
            datos
                .pointer("/declared/systemLanguage")
                .and_then(Value::as_str)
        })
        .unwrap_or("es");
    formatos_idioma.set_text(formato_idioma_actual);
    grupo.add(&formatos_idioma);

    let formatos_region = adw::EntryRow::new();
    formatos_region.set_title(&localizar_visible(idioma_actual(), "Región de formatos"));
    let formato_region_actual = datos
        .pointer("/declared/formats/region")
        .and_then(Value::as_str)
        .or_else(|| datos.pointer("/declared/region").and_then(Value::as_str))
        .unwrap_or("PE");
    formatos_region.set_text(formato_region_actual);
    grupo.add(&formatos_region);

    let zona = adw::EntryRow::new();
    zona.set_title(&localizar_visible(idioma_actual(), "Zona horaria"));
    zona.set_text(&valor(datos, "/declared/timeZone"));
    grupo.add(&zona);

    let teclado = adw::EntryRow::new();
    teclado.set_title(&localizar_visible(
        idioma_actual(),
        "Distribución de teclado",
    ));
    let teclado_actual = datos
        .pointer("/declared/keyboard/layout")
        .and_then(Value::as_str)
        .or_else(|| {
            datos
                .pointer("/derived/keyboard/layout")
                .and_then(Value::as_str)
        })
        .unwrap_or("es");
    teclado.set_text(teclado_actual);
    grupo.add(&teclado);

    let variante = adw::EntryRow::new();
    variante.set_title(&localizar_visible(idioma_actual(), "Variante"));
    variante.set_text(
        datos
            .pointer("/declared/keyboard/variant")
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    grupo.add(&variante);

    let fila_guardar = adw::ActionRow::new();
    fila_guardar.set_title(&localizar_visible(idioma_actual(), "Guardar localización"));
    fila_guardar.set_subtitle(&localizar_visible(
        idioma_actual(),
        "Korunix valida los códigos y la configuración antes de aplicarla.",
    ));
    let guardar = gtk::Button::with_label(texto(estado.idioma, "save_apply"));
    guardar.add_css_class("suggested-action");
    guardar.set_valign(gtk::Align::Center);
    fila_guardar.add_suffix(&guardar);
    grupo.add(&fila_guardar);

    pagina.add(&grupo);

    let grupo_personalizar = grupo.clone();
    personalizar.connect_clicked(move |boton| {
        let visible = !grupo_personalizar.is_visible();
        grupo_personalizar.set_visible(visible);
        boton.set_label(&localizar_visible(
            idioma_actual(),
            if visible { "Ocultar" } else { "Personalizar" },
        ));
    });

    let estado_guardar = Rc::clone(&estado);
    guardar.connect_clicked(move |boton| {
        let args = vec![
            "localization".to_string(),
            "set".to_string(),
            "--language".to_string(),
            idioma.text().trim().to_string(),
            "--region".to_string(),
            region.text().trim().to_string(),
            "--formats-language".to_string(),
            formatos_idioma.text().trim().to_string(),
            "--formats-region".to_string(),
            formatos_region.text().trim().to_string(),
            "--timezone".to_string(),
            zona.text().trim().to_string(),
            "--keyboard".to_string(),
            teclado.text().trim().to_string(),
            "--variant".to_string(),
            variante.text().trim().to_string(),
            "--plan".to_string(),
            "--json".to_string(),
        ];

        if let Err(error) = ejecutar_json_owned(&estado_guardar, &args) {
            mostrar_error(&estado_guardar, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_guardar,
            "¿Guardar y aplicar estas decisiones de idioma, región y teclado?",
            texto(estado_guardar.idioma, "save_apply"),
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_guardar);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let mut ejecutar = args.clone();
            ejecutar.retain(|v| v != "--plan");
            ejecutar.insert(ejecutar.len() - 1, "--yes".to_string());

            match ejecutar_json_owned(&estado_ejecutar, &ejecutar)
                .and_then(|_| aplicar_configuracion_gui(&estado_ejecutar))
            {
                Ok(_) => {
                    mostrar_exito(
                        &estado_ejecutar,
                        texto(estado_ejecutar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });

        dialogo.present();
    });

    pagina
}

fn pagina_personas(estado: Rc<Estado>, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let grupo_actual = adw::PreferencesGroup::new();
    grupo_actual.set_title(&localizar_visible(
        idioma_actual(),
        "Personas de este equipo",
    ));

    let cuentas = datos
        .pointer("/accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if cuentas.is_empty() {
        grupo_actual.add(&fila("Estado", texto(estado.idioma, "empty")));
    } else {
        for cuenta in cuentas {
            let nombre = cuenta
                .get("displayName")
                .and_then(Value::as_str)
                .or_else(|| cuenta.get("accountName").and_then(Value::as_str))
                .unwrap_or("—");

            let cuenta_id = cuenta
                .get("accountName")
                .and_then(Value::as_str)
                .unwrap_or("—");

            let status = cuenta.get("status").and_then(Value::as_str).unwrap_or("");
            let subtitulo = if status.is_empty() {
                cuenta_id.to_string()
            } else {
                format!(
                    "{cuenta_id} · {}",
                    estado_cuenta_humano(estado.idioma, status),
                )
            };

            grupo_actual.add(&fila(nombre, subtitulo));
        }
    }

    pagina.add(&grupo_actual);

    let grupo_nueva = adw::PreferencesGroup::new();
    grupo_nueva.set_title(texto(estado.idioma, "create_person"));
    grupo_nueva.set_description(Some(
        "La contraseña se entrega al sistema después de crear la cuenta y nunca se guarda en el repositorio ni en el historial.",
    ));

    let nombre = adw::EntryRow::new();
    nombre.set_title(&localizar_visible(idioma_actual(), "Nombre visible"));
    grupo_nueva.add(&nombre);

    let cuenta = adw::EntryRow::new();
    cuenta.set_title(&localizar_visible(idioma_actual(), "Nombre de cuenta"));
    grupo_nueva.add(&cuenta);

    let password = adw::PasswordEntryRow::new();
    password.set_title(&localizar_visible(idioma_actual(), "Contraseña"));
    grupo_nueva.add(&password);

    let confirmar = adw::PasswordEntryRow::new();
    confirmar.set_title(&localizar_visible(idioma_actual(), "Confirmar contraseña"));
    grupo_nueva.add(&confirmar);

    let avatar = adw::EntryRow::new();
    avatar.set_title(&localizar_visible(
        idioma_actual(),
        "Avatar opcional · ruta PNG/JPEG/WebP",
    ));
    grupo_nueva.add(&avatar);

    let roles = gtk::StringList::new(&["Estándar", "Administrador"]);
    let rol = adw::ComboRow::new();
    rol.set_title(&localizar_visible(idioma_actual(), "Rol"));
    rol.set_model(Some(&roles));
    rol.set_selected(0);
    grupo_nueva.add(&rol);

    let fila_crear = adw::ActionRow::new();
    fila_crear.set_title(&localizar_visible(idioma_actual(), "Crear cuenta"));
    let crear = gtk::Button::with_label(texto(estado.idioma, "create_person"));
    crear.add_css_class("suggested-action");
    crear.set_valign(gtk::Align::Center);
    fila_crear.add_suffix(&crear);
    grupo_nueva.add(&fila_crear);

    pagina.add(&grupo_nueva);

    let estado_crear = Rc::clone(&estado);
    crear.connect_clicked(move |boton| {
        let account = cuenta.text().trim().to_string();
        let display = nombre.text().trim().to_string();
        let secreto = password.text().to_string();
        let repetido = confirmar.text().to_string();
        let avatar_path = avatar.text().trim().to_string();
        let role = if rol.selected() == 1 {
            "admin"
        } else {
            "standard"
        };

        if secreto.is_empty() {
            mostrar_error(&estado_crear, "La contraseña no puede estar vacía.");
            return;
        }

        if secreto != repetido {
            mostrar_error(&estado_crear, "Las contraseñas no coinciden.");
            return;
        }

        let mut plan = vec![
            "users".to_string(),
            "create".to_string(),
            "--account".to_string(),
            account.clone(),
            "--name".to_string(),
            display.clone(),
            "--role".to_string(),
            role.to_string(),
        ];

        if !avatar_path.is_empty() {
            plan.extend(["--avatar".to_string(), avatar_path.clone()]);
        }

        plan.extend(["--plan".to_string(), "--json".to_string()]);

        if let Err(error) = ejecutar_json_owned(&estado_crear, &plan) {
            mostrar_error(&estado_crear, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_crear,
            &format!("¿Crear la cuenta «{account}» y aplicar la configuración?"),
            texto(estado_crear.idioma, "create_person"),
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_crear);
        let password_limpiar = password.clone();
        let confirmar_limpiar = confirmar.clone();
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let resultado = (|| -> Result<Value, String> {
                let mut ejecutar = plan.clone();
                ejecutar.retain(|v| v != "--plan");
                ejecutar.insert(ejecutar.len() - 1, "--yes".to_string());

                ejecutar_json_owned(&estado_ejecutar, &ejecutar)?;
                aplicar_configuracion_gui(&estado_ejecutar)?;

                ejecutar_json_con_stdin(
                    &estado_ejecutar,
                    &[
                        "users",
                        "password-stdin",
                        account.as_str(),
                        "--yes",
                        "--json",
                    ],
                    &(secreto.clone() + "\n"),
                )
            })();

            match resultado {
                Ok(_) => {
                    password_limpiar.set_text("");
                    confirmar_limpiar.set_text("");
                    mostrar_exito(
                        &estado_ejecutar,
                        "Persona creada. La contraseña no fue guardada por Korunix.",
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });

        dialogo.present();
    });

    pagina
}

fn id_generacion_plan(valor: &Value) -> Option<u32> {
    if let Some(id) = valor.as_u64() {
        return u32::try_from(id).ok();
    }

    if let Some(id) = valor.as_str().and_then(|valor| valor.parse::<u32>().ok()) {
        return Some(id);
    }

    for clave in ["id", "generation", "generationId"] {
        if let Some(id) = valor.get(clave).and_then(|valor| {
            valor
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .or_else(|| valor.as_str().and_then(|id| id.parse::<u32>().ok()))
        }) {
            return Some(id);
        }
    }

    None
}

fn generaciones_conservadas(plan: &Value) -> Vec<u32> {
    let mut ids = plan
        .get("keep")
        .and_then(Value::as_array)
        .map(|elementos| {
            elementos
                .iter()
                .filter_map(id_generacion_plan)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ids.sort_unstable();
    ids.dedup();
    ids
}

fn texto_version_actual(idioma: Idioma, conservada: bool) -> String {
    match (idioma, conservada) {
        (Idioma::Ingles, true) => "Current · Kept for recovery".to_string(),
        (Idioma::BelarusLatino, true) => {
            "Cjapjerašnjaje · Zachoŭvajecca dłja adnaŭłjennja".to_string()
        }
        (Idioma::Belarus, true) => "Бягучы · Захоўваецца для аднаўлення".to_string(),
        (Idioma::Catalan, true) => "Actual · Mantingut per a la recuperació".to_string(),
        (Idioma::Checo, true) => "Aktuální · Uchováváno pro obnovení".to_string(),
        (Idioma::Aleman, true) => "Aktuell · Zur Wiederherstellung aufbewahrt".to_string(),
        (Idioma::Frances, true) => "Actuel · Conservé pour récupération".to_string(),
        (Idioma::Gallego, true) => "Actual · Mantívose para a súa recuperación".to_string(),
        (Idioma::Italiano, true) => "Corrente · Conservato per il recupero".to_string(),
        (Idioma::Coreano, true) => "현재 · 복구를 위해 보관".to_string(),
        (Idioma::Kurdo, true) => "Niha · Ji bo vegirtinê tê parastin".to_string(),
        (Idioma::Neerlandes, true) => "Huidig · Bewaarde voor herstel".to_string(),
        (Idioma::NoruegoNynorsk, true) => "Gjeldende · Beholdt for gjenoppretting".to_string(),
        (Idioma::Polaco, true) => "Aktualny · Zachowany do odzyskania".to_string(),
        (Idioma::PortuguesBrasil, true) => "Atual · Mantido para recuperação".to_string(),
        (Idioma::Ruso, true) => "Текущий · Сохранено для восстановления".to_string(),
        (Idioma::Sueco, true) => "Aktuell · Sparas för återställning".to_string(),
        (Idioma::Turco, true) => "Current · Kept for recovery".to_string(),
        (Idioma::Ucraniano, true) => "Поточний · Зберігається для відновлення".to_string(),
        (Idioma::Vietnamita, true) => "Hiện tại · Giữ để phục hồi".to_string(),
        (Idioma::ChinoSimplificado, true) => "当前 · 保留用于恢复".to_string(),
        (Idioma::Ingles, false) => "Current".to_string(),
        (Idioma::BelarusLatino, false) => "Tok".to_string(),
        (Idioma::Belarus, false) => "Ток".to_string(),
        (Idioma::Catalan, false) => "Actual".to_string(),
        (Idioma::Checo, false) => "Aktuální".to_string(),
        (Idioma::Aleman, false) => "Aktuell".to_string(),
        (Idioma::Frances, false) => "Actuel".to_string(),
        (Idioma::Gallego, false) => "Actual".to_string(),
        (Idioma::Italiano, false) => "Corrente".to_string(),
        (Idioma::Coreano, false) => "현재".to_string(),
        (Idioma::Kurdo, false) => "Niha".to_string(),
        (Idioma::Neerlandes, false) => "Huidig".to_string(),
        (Idioma::NoruegoNynorsk, false) => "Gjeldende".to_string(),
        (Idioma::Polaco, false) => "Prąd".to_string(),
        (Idioma::PortuguesBrasil, false) => "Atual".to_string(),
        (Idioma::Ruso, false) => "Текущий".to_string(),
        (Idioma::Sueco, false) => "Aktuell".to_string(),
        (Idioma::Turco, false) => "Güncel".to_string(),
        (Idioma::Ucraniano, false) => "Поточний".to_string(),
        (Idioma::Vietnamita, false) => "hiện tại".to_string(),
        (Idioma::ChinoSimplificado, false) => "当前".to_string(),
        (Idioma::Hungaro, true) => "Jelenlegi · Helyreállításhoz megtartva".to_string(),
        (Idioma::Hungaro, false) => "Jelenlegi".to_string(),
        (Idioma::Espanol, true) => "Actual · Conservada para recuperación".to_string(),
        (Idioma::Espanol, false) => "Actual".to_string(),
    }
}

fn etiqueta_version_recuperacion(
    idioma: Idioma,
    numero_anterior: usize,
    actual: bool,
    inicio: bool,
    conservada: bool,
) -> String {
    if actual {
        return match idioma {
            Idioma::Ingles => "Current · Kept".to_string(),
            Idioma::BelarusLatino => "Bjahučy · Zachavany".to_string(),
            Idioma::Belarus => "Актуальны · Захаваны".to_string(),
            Idioma::Catalan => "Actual · Mantingut".to_string(),
            Idioma::Checo => "Aktuální · Zachováno".to_string(),
            Idioma::Aleman => "Aktuell · Behalten".to_string(),
            Idioma::Frances => "Actuel · Conservé".to_string(),
            Idioma::Gallego => "Actual · Mantívose".to_string(),
            Idioma::Italiano => "Attuale · Mantenuto".to_string(),
            Idioma::Coreano => "현재 · 유지".to_string(),
            Idioma::Kurdo => "Niha · Ketî".to_string(),
            Idioma::Neerlandes => "Huidig · Behouden".to_string(),
            Idioma::NoruegoNynorsk => "Gjeldende · Beholdt".to_string(),
            Idioma::Polaco => "Aktualny · Zachowany".to_string(),
            Idioma::PortuguesBrasil => "Atual · Mantido".to_string(),
            Idioma::Ruso => "Текущий · Сохранено".to_string(),
            Idioma::Sueco => "Aktuell · Bevarad".to_string(),
            Idioma::Turco => "Güncel · Tutulan".to_string(),
            Idioma::Ucraniano => "Поточний · Зберігається".to_string(),
            Idioma::Vietnamita => "Hiện tại · Giữ".to_string(),
            Idioma::ChinoSimplificado => "当前 · 保留".to_string(),
            Idioma::Hungaro => "Jelenlegi · Megtartva".to_string(),
            Idioma::Espanol => "Actual · Conservada".to_string(),
        };
    }

    let base = if inicio {
        match idioma {
            Idioma::Ingles => "Startup".to_string(),
            Idioma::BelarusLatino => "Zapusk".to_string(),
            Idioma::Belarus => "Запуск".to_string(),
            Idioma::Catalan => "Inici".to_string(),
            Idioma::Checo => "Spuštění".to_string(),
            Idioma::Aleman => "Start".to_string(),
            Idioma::Frances => "Démarrage".to_string(),
            Idioma::Gallego => "Inicio".to_string(),
            Idioma::Italiano => "Avvio".to_string(),
            Idioma::Coreano => "시작".to_string(),
            Idioma::Kurdo => "Destpêk".to_string(),
            Idioma::Neerlandes => "Opstarten".to_string(),
            Idioma::NoruegoNynorsk => "Oppstart".to_string(),
            Idioma::Polaco => "Uruchamianie".to_string(),
            Idioma::PortuguesBrasil => "Inicialização".to_string(),
            Idioma::Ruso => "Запуск".to_string(),
            Idioma::Sueco => "Start".to_string(),
            Idioma::Turco => "Başlangıç".to_string(),
            Idioma::Ucraniano => "Запуск".to_string(),
            Idioma::Vietnamita => "Khởi động".to_string(),
            Idioma::ChinoSimplificado => "启动".to_string(),
            Idioma::Hungaro => "Indítás".to_string(),
            Idioma::Espanol => "Inicio".to_string(),
        }
    } else {
        match (idioma, numero_anterior) {
            (Idioma::Ingles, 1) => "Previous".to_string(),
            (Idioma::BelarusLatino, 1) => "Papjaredni".to_string(),
            (Idioma::Belarus, 1) => "Папярэдняя".to_string(),
            (Idioma::Catalan, 1) => "Anterior".to_string(),
            (Idioma::Checo, 1) => "Předchozí".to_string(),
            (Idioma::Aleman, 1) => "Zurück".to_string(),
            (Idioma::Frances, 1) => "Précédent".to_string(),
            (Idioma::Gallego, 1) => "Anterior".to_string(),
            (Idioma::Italiano, 1) => "Precedente".to_string(),
            (Idioma::Coreano, 1) => "이전".to_string(),
            (Idioma::Kurdo, 1) => "Berê".to_string(),
            (Idioma::Neerlandes, 1) => "Vorige".to_string(),
            (Idioma::NoruegoNynorsk, 1) => "Forrige".to_string(),
            (Idioma::Polaco, 1) => "Poprzedni".to_string(),
            (Idioma::PortuguesBrasil, 1) => "Anterior".to_string(),
            (Idioma::Ruso, 1) => "Предыдущий".to_string(),
            (Idioma::Sueco, 1) => "Föregående".to_string(),
            (Idioma::Turco, 1) => "Önceki".to_string(),
            (Idioma::Ucraniano, 1) => "Попередній".to_string(),
            (Idioma::Vietnamita, 1) => "Trước".to_string(),
            (Idioma::ChinoSimplificado, 1) => "上一页".to_string(),
            (Idioma::Ingles, n) => format!("{n} back"),
            (Idioma::BelarusLatino, n) => format!("{n} nazad"),
            (Idioma::Belarus, n) => format!("{n} назад"),
            (Idioma::Catalan, n) => format!("{n} tornar"),
            (Idioma::Checo, n) => format!("{n} zpět"),
            (Idioma::Aleman, n) => format!("{n} zurück"),
            (Idioma::Frances, n) => format!("{n} retour"),
            (Idioma::Gallego, n) => format!("{n} atrás"),
            (Idioma::Italiano, n) => format!("{n} indietro"),
            (Idioma::Coreano, n) => format!("{n} 뒤로"),
            (Idioma::Kurdo, n) => format!("{n} paş"),
            (Idioma::Neerlandes, n) => format!("{n} terug"),
            (Idioma::NoruegoNynorsk, n) => format!("{n} tilbake"),
            (Idioma::Polaco, n) => format!("{n} z powrotem"),
            (Idioma::PortuguesBrasil, n) => format!("{n} voltar"),
            (Idioma::Ruso, n) => format!("{n} назад"),
            (Idioma::Sueco, n) => format!("{n} tillbaka"),
            (Idioma::Turco, n) => format!("{n} geri"),
            (Idioma::Ucraniano, n) => format!("{n} назад"),
            (Idioma::Vietnamita, n) => format!("{n} quay lại"),
            (Idioma::ChinoSimplificado, n) => format!("{n} 返回"),
            (Idioma::Hungaro, 1) => "Előző".to_string(),
            (Idioma::Hungaro, n) => format!("{n} vissza"),
            (Idioma::Espanol, 1) => "Anterior".to_string(),
            (Idioma::Espanol, n) => format!("{n} atrás"),
        }
    };

    if conservada {
        match idioma {
            Idioma::Ingles => format!("{base} · Kept"),
            Idioma::BelarusLatino => format!("{base} · Zachavana"),
            Idioma::Belarus => format!("{base} · Захавана"),
            Idioma::Catalan => format!("{base} · Mantingut"),
            Idioma::Checo => format!("{base} · Zachováno"),
            Idioma::Aleman => format!("{base} · Behalten"),
            Idioma::Frances => format!("{base} · Conservé"),
            Idioma::Gallego => format!("{base} · Mantívose"),
            Idioma::Italiano => format!("{base} · Mantenuto"),
            Idioma::Coreano => format!("{base} · 유지됨"),
            Idioma::Kurdo => format!("{base} · Parastiye"),
            Idioma::Neerlandes => format!("{base} · Behouden"),
            Idioma::NoruegoNynorsk => format!("{base} · Beholdt"),
            Idioma::Polaco => format!("{base} · Zachowane"),
            Idioma::PortuguesBrasil => format!("{base} · Mantido"),
            Idioma::Ruso => format!("{base} · Сохранено"),
            Idioma::Sueco => format!("{base} · Behålls"),
            Idioma::Turco => format!("{base} · Tutuldu"),
            Idioma::Ucraniano => format!("{base} · Збережено"),
            Idioma::Vietnamita => format!("{base} · Được giữ"),
            Idioma::ChinoSimplificado => format!("{base} · 保留"),
            Idioma::Hungaro => format!("{base} · Megtartva"),
            Idioma::Espanol => format!("{base} · Conservada"),
        }
    } else {
        match idioma {
            Idioma::Ingles => format!("{base} · Removed"),
            Idioma::BelarusLatino => format!("{base} · Vydałjena"),
            Idioma::Belarus => format!("{base} · Выдалена"),
            Idioma::Catalan => format!("{base} · Eliminat"),
            Idioma::Checo => format!("{base} · Odebráno"),
            Idioma::Aleman => format!("{base} · Entfernt"),
            Idioma::Frances => format!("{base} · Supprimé"),
            Idioma::Gallego => format!("{base} · Eliminado"),
            Idioma::Italiano => format!("{base} · Rimosso"),
            Idioma::Coreano => format!("{base} · 삭제됨"),
            Idioma::Kurdo => format!("{base} · Rakir"),
            Idioma::Neerlandes => format!("{base} · Verwijderd"),
            Idioma::NoruegoNynorsk => format!("{base} · Fjernet"),
            Idioma::Polaco => format!("{base} · Usunięto"),
            Idioma::PortuguesBrasil => format!("{base} · Removido"),
            Idioma::Ruso => format!("{base} · Удален"),
            Idioma::Sueco => format!("{base} · Borttagen"),
            Idioma::Turco => format!("{base} · Kaldırıldı"),
            Idioma::Ucraniano => format!("{base} · Видалено"),
            Idioma::Vietnamita => format!("{base} · Đã xóa"),
            Idioma::ChinoSimplificado => format!("{base} · 已删除"),
            Idioma::Hungaro => format!("{base} · Törlődik"),
            Idioma::Espanol => format!("{base} · Se elimina"),
        }
    }
}

fn fila_recuperacion(
    estado: Rc<Estado>,
    id: u32,
    titulo: String,
    detalle: String,
    actual: bool,
) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(&localizar_visible(idioma_actual(), &titulo));
    row.set_subtitle(&localizar_visible(idioma_actual(), &detalle));

    if !actual {
        let boton = gtk::Button::with_label(texto(estado.idioma, "use_once"));
        boton.set_valign(gtk::Align::Center);
        row.add_suffix(&boton);

        let estado_recuperacion = Rc::clone(&estado);
        boton.connect_clicked(move |boton| {
            let id_texto = id.to_string();

            if let Err(error) = ejecutar_json(
                &estado_recuperacion,
                &["rollback", &id_texto, "--plan", "--json"],
            ) {
                mostrar_error(&estado_recuperacion, error);
                return;
            }

            let cuerpo = frase_confirmar_recuperacion(estado_recuperacion.idioma, id);
            let dialogo = dialogo_confirmacion(
                boton,
                estado_recuperacion.idioma,
                &cuerpo,
                texto(estado_recuperacion.idioma, "use_once"),
                false,
            );

            let estado_aplicar = Rc::clone(&estado_recuperacion);
            dialogo.connect_response(None, move |_, respuesta| {
                if respuesta != "apply" {
                    return;
                }

                match ejecutar_json(&estado_aplicar, &["rollback", &id_texto, "--yes", "--json"]) {
                    Ok(_) => {
                        mostrar_exito(
                            &estado_aplicar,
                            texto(estado_aplicar.idioma, "operation_done"),
                        );
                        recargar(Rc::clone(&estado_aplicar));
                    }
                    Err(error) => mostrar_error(&estado_aplicar, error),
                }
            });

            dialogo.present();
        });
    }

    row
}

fn conectar_limpieza(estado: Rc<Estado>, boton: &gtk::Button, agresiva: bool, eliminables: usize) {
    let boton = boton.clone();

    boton.connect_clicked(move |boton| {
        let cuerpo = if agresiva {
            format!(
                "¿Hacer una limpieza profunda? Korunix conservará las versiones protegidas y retirará {eliminables} versiones antiguas que no lo están."
            )
        } else {
            format!(
                "¿Limpiar el sistema? Korunix conservará los puntos de recuperación recientes y retirará {eliminables} versiones antiguas."
            )
        };

        let dialogo = dialogo_accion(
            boton,
            &estado,
            &cuerpo,
            if agresiva {
                "Limpieza profunda"
            } else {
                "Limpiar"
            },
            agresiva,
        );

        let estado_aplicar = Rc::clone(&estado);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let argumentos = if agresiva {
                ["clean-all", "--yes", "--json"]
            } else {
                ["clean", "--yes", "--json"]
            };

            match ejecutar_json(&estado_aplicar, &argumentos) {
                Ok(_) => {
                    mostrar_exito(
                        &estado_aplicar,
                        texto(estado_aplicar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_aplicar));
                }
                Err(error) => mostrar_error(&estado_aplicar, error),
            }
        });

        dialogo.present();
    });
}

fn pagina_mantenimiento(
    estado: Rc<Estado>,
    recuperacion: &Value,
    limpieza: &Value,
    limpieza_total: &Value,
    privilegios: &Value,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let grupo_recuperacion = adw::PreferencesGroup::new();
    grupo_recuperacion.set_title(&localizar_visible(
        idioma_actual(),
        "Versiones para recuperación",
    ));
    grupo_recuperacion.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Korunix muestra las tres versiones recientes más útiles. Puedes preparar una anterior para el próximo arranque sin cambiar la sesión actual.",
    )));

    let predeterminada = recuperacion
        .get("defaultGeneration")
        .and_then(Value::as_u64)
        .and_then(|id| u32::try_from(id).ok());

    let conservadas = generaciones_conservadas(limpieza);
    let generaciones = recuperacion
        .get("generations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut mostradas = 0usize;
    let mut anteriores = 0usize;

    for generacion in generaciones.iter().rev() {
        if mostradas >= 3 {
            break;
        }

        let Some(id) = generacion
            .get("id")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok())
        else {
            continue;
        };

        let actual = generacion.get("current").and_then(Value::as_bool) == Some(true);
        let inicio = Some(id) == predeterminada;

        if !actual {
            anteriores += 1;
        }

        let titulo = if actual {
            "Sistema actual".to_string()
        } else if inicio {
            "Inicio predeterminado".to_string()
        } else if anteriores == 1 {
            "Versión anterior".to_string()
        } else {
            format!("Versión anterior {anteriores}")
        };

        let version = generacion
            .get("nixosVersion")
            .and_then(Value::as_str)
            .filter(|valor| !valor.trim().is_empty())
            .unwrap_or("NixOS");

        let protegida = actual || inicio || conservadas.contains(&id);
        let detalle = if protegida {
            format!("{version} · Conservada para recuperación")
        } else {
            format!("{version} · Disponible hasta la próxima limpieza")
        };

        grupo_recuperacion.add(&fila_recuperacion(
            Rc::clone(&estado),
            id,
            titulo,
            detalle,
            actual,
        ));
        mostradas += 1;
    }

    if mostradas == 0 {
        grupo_recuperacion.add(&fila(
            "Estado",
            "Korunix no encontró versiones de NixOS para recuperación.",
        ));
    }

    pagina.add(&grupo_recuperacion);

    let grupo_limpieza = adw::PreferencesGroup::new();
    grupo_limpieza.set_title(texto(estado.idioma, "cleanup"));
    grupo_limpieza.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Korunix puede contar las versiones que retirará, pero Nix no informa con precisión cuánto espacio se liberará antes de ejecutar la limpieza.",
    )));

    let normal_eliminar = limpieza
        .get("delete")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let normal_conservar = limpieza
        .get("keep")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    let normal = adw::ActionRow::new();
    normal.set_title(&localizar_visible(idioma_actual(), "Limpieza recomendada"));
    normal.set_subtitle(&localizar_visible(
        idioma_actual(),
        &format!(
            "Conserva {normal_conservar} versiones útiles para recuperación y puede retirar {normal_eliminar} versiones antiguas. También recoge referencias sin uso y optimiza el almacén de Nix."
        ),
    ));
    let limpiar = gtk::Button::with_label(&localizar_visible(idioma_actual(), "Limpiar"));
    limpiar.add_css_class("suggested-action");
    limpiar.set_valign(gtk::Align::Center);
    normal.add_suffix(&limpiar);
    grupo_limpieza.add(&normal);

    let profunda_eliminar = limpieza_total
        .get("delete")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let profunda_conservar = limpieza_total
        .get("keep")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    let profunda = adw::ActionRow::new();
    profunda.set_title(&localizar_visible(idioma_actual(), "Limpieza profunda"));
    profunda.set_subtitle(&localizar_visible(
        idioma_actual(),
        &format!(
            "Conserva {profunda_conservar} versiones protegidas y puede retirar {profunda_eliminar} versiones antiguas adicionales."
        ),
    ));
    let limpiar_profundo =
        gtk::Button::with_label(&localizar_visible(idioma_actual(), "Limpieza profunda"));
    limpiar_profundo.add_css_class("destructive-action");
    limpiar_profundo.set_valign(gtk::Align::Center);
    profunda.add_suffix(&limpiar_profundo);
    grupo_limpieza.add(&profunda);

    conectar_limpieza(Rc::clone(&estado), &limpiar, false, normal_eliminar);
    conectar_limpieza(
        Rc::clone(&estado),
        &limpiar_profundo,
        true,
        profunda_eliminar,
    );

    pagina.add(&grupo_limpieza);

    if privilegios.get("guiUsable").and_then(Value::as_bool) == Some(false) {
        let grupo_privilegios = adw::PreferencesGroup::new();
        grupo_privilegios.set_title(&localizar_visible(idioma_actual(), "Permisos"));
        grupo_privilegios.add(&fila(
            "Korunix necesita atención",
            "La interfaz no puede solicitar los permisos necesarios para aplicar cambios del sistema.",
        ));
        pagina.add(&grupo_privilegios);
    }

    pagina
}

fn tamano_almacenamiento_humano(valor: &str) -> String {
    let valor = valor.trim();

    if valor.len() >= 2 {
        let (numero, sufijo) = valor.split_at(valor.len() - 1);

        if matches!(sufijo, "K" | "M" | "G" | "T" | "P")
            && numero
                .chars()
                .all(|caracter| caracter.is_ascii_digit() || matches!(caracter, ',' | '.'))
        {
            return format!("{numero} {sufijo}B");
        }
    }

    valor.to_string()
}

fn pagina_almacenamiento(estado: Rc<Estado>, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let dispositivos = datos
        .get("devices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let hay_extraibles = dispositivos.iter().any(|dispositivo| {
        dispositivo
            .get("removable")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    let pesada = adw::SwitchRow::new();
    pesada.set_title(&localizar_visible(
        idioma_actual(),
        "Expulsión después de archivos grandes",
    ));
    pesada.set_subtitle(&localizar_visible(
        idioma_actual(),
        "Actívalo antes de expulsar una unidad si acabas de copiar una ISO u otros archivos grandes. Korunix esperará a que los datos pendientes terminen de escribirse.",
    ));

    if hay_extraibles {
        let grupo_seguridad = adw::PreferencesGroup::new();
        grupo_seguridad.set_title(&localizar_visible(idioma_actual(), "Expulsión segura"));
        grupo_seguridad.set_description(Some(&localizar_visible(
            idioma_actual(),
            "La unidad solo se marcará como segura para desconectar después de desmontarla y apagarla correctamente.",
        )));
        grupo_seguridad.add(&pesada);
        pagina.add(&grupo_seguridad);
    }

    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(&localizar_visible(idioma_actual(), "Unidades"));

    if dispositivos.is_empty() {
        grupo.add(&fila(
            "Estado",
            "Korunix no encontró unidades de almacenamiento.",
        ));
    }

    for dispositivo in dispositivos {
        let ruta = dispositivo
            .get("device")
            .and_then(Value::as_str)
            .unwrap_or("—")
            .to_string();

        let modelo = dispositivo
            .get("model")
            .and_then(Value::as_str)
            .filter(|valor| !valor.trim().is_empty())
            .unwrap_or(&ruta)
            .to_string();

        let tamano = tamano_almacenamiento_humano(
            dispositivo
                .get("size")
                .and_then(Value::as_str)
                .unwrap_or("—"),
        );

        let extraible = dispositivo
            .get("removable")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let montajes = dispositivo
            .get("mountPoints")
            .and_then(Value::as_array)
            .map(|puntos| {
                puntos
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|punto| !punto.trim().is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let estado_unidad = if extraible {
            if montajes.is_empty() {
                "Extraíble · No está montada"
            } else {
                "Extraíble · Disponible ahora"
            }
        } else if montajes.is_empty() {
            "Interna"
        } else {
            "Interna · Disponible ahora"
        };

        let row = adw::ActionRow::new();
        row.set_title(&modelo);
        row.set_subtitle(&localizar_visible(
            idioma_actual(),
            &format!("{tamano} · {estado_unidad}"),
        ));

        if extraible {
            let boton = gtk::Button::with_label(texto(estado.idioma, "eject"));
            boton.set_valign(gtk::Align::Center);
            row.add_suffix(&boton);

            let estado_expulsion = Rc::clone(&estado);
            let pesada_expulsion = pesada.clone();
            let ruta_expulsion = ruta.clone();

            boton.connect_clicked(move |boton| {
                let modo_pesado = pesada_expulsion.is_active();

                let plan = if modo_pesado {
                    ejecutar_json(
                        &estado_expulsion,
                        &[
                            "storage",
                            "eject",
                            &ruta_expulsion,
                            "--heavy",
                            "--plan",
                            "--json",
                        ],
                    )
                } else {
                    ejecutar_json(
                        &estado_expulsion,
                        &["storage", "eject", &ruta_expulsion, "--plan", "--json"],
                    )
                };

                if let Err(error) = plan {
                    mostrar_error(&estado_expulsion, error);
                    return;
                }

                let cuerpo = if modo_pesado {
                    "¿Expulsar esta unidad? Korunix esperará primero a que terminen de escribirse los datos pendientes y solo entonces la apagará."
                } else {
                    "¿Expulsar esta unidad de forma segura? Korunix la desmontará y apagará antes de indicar que puede desconectarse."
                };

                let dialogo = dialogo_confirmacion(
                    boton,
                    estado_expulsion.idioma,
                    cuerpo,
                    texto(estado_expulsion.idioma, "eject"),
                    false,
                );

                let estado_aplicar = Rc::clone(&estado_expulsion);
                let ruta_aplicar = ruta_expulsion.clone();

                dialogo.connect_response(None, move |_, respuesta| {
                    if respuesta != "apply" {
                        return;
                    }

                    let resultado = if modo_pesado {
                        ejecutar_json(
                            &estado_aplicar,
                            &[
                                "storage",
                                "eject",
                                &ruta_aplicar,
                                "--heavy",
                                "--yes",
                                "--json",
                            ],
                        )
                    } else {
                        ejecutar_json(
                            &estado_aplicar,
                            &["storage", "eject", &ruta_aplicar, "--yes", "--json"],
                        )
                    };

                    match resultado {
                        Ok(_) => {
                            mostrar_exito(
                                &estado_aplicar,
                                texto(estado_aplicar.idioma, "safe_disconnect"),
                            );
                            recargar(Rc::clone(&estado_aplicar));
                        }
                        Err(error) => mostrar_error(&estado_aplicar, error),
                    }
                });

                dialogo.present();
            });
        }

        grupo.add(&row);
    }

    pagina.add(&grupo);
    pagina
}

fn pagina_firmware(
    estado: Rc<Estado>,
    dispositivos: &Value,
    actualizaciones: &Value,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let updates = actualizaciones
        .get("devices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let devices = dispositivos
        .get("devices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let problemas = devices
        .iter()
        .filter(|device| {
            device
                .get("problems")
                .and_then(Value::as_array)
                .map(|problemas| !problemas.is_empty())
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();

    let grupo_estado = adw::PreferencesGroup::new();
    grupo_estado.set_title(&localizar_visible(idioma_actual(), "Estado del firmware"));

    let fila_estado = adw::ActionRow::new();
    if updates.is_empty() && problemas.is_empty() {
        fila_estado.set_title(&localizar_visible(
            idioma_actual(),
            "El firmware está al día",
        ));
        fila_estado.set_subtitle(&localizar_visible(
            idioma_actual(),
            "No hay actualizaciones ni dispositivos de firmware que requieran atención.",
        ));
    } else if !updates.is_empty() {
        fila_estado.set_title(&localizar_visible(
            idioma_actual(),
            if updates.len() == 1 {
                "Hay una actualización de firmware"
            } else {
                "Hay actualizaciones de firmware"
            },
        ));
        fila_estado.set_subtitle(&localizar_visible(
            idioma_actual(),
            &format!(
                "{} {} disponibles.",
                updates.len(),
                if updates.len() == 1 {
                    "actualización"
                } else {
                    "actualizaciones"
                }
            ),
        ));
    } else {
        fila_estado.set_title(&localizar_visible(
            idioma_actual(),
            "Hay dispositivos de firmware que requieren atención",
        ));
        fila_estado.set_subtitle(&localizar_visible(
            idioma_actual(),
            "No hay una actualización disponible ahora, pero fwupd informó de un problema.",
        ));
    }

    let comprobar =
        gtk::Button::with_label(&localizar_visible(idioma_actual(), "Comprobar de nuevo"));
    comprobar.set_valign(gtk::Align::Center);
    fila_estado.add_suffix(&comprobar);
    grupo_estado.add(&fila_estado);
    pagina.add(&grupo_estado);

    let estado_buscar = Rc::clone(&estado);
    comprobar.connect_clicked(move |_| {
        match ejecutar_json(&estado_buscar, &["firmware", "refresh", "--yes", "--json"]) {
            Ok(_) => {
                mostrar_exito(&estado_buscar, "La información de firmware se actualizó.");
                recargar(Rc::clone(&estado_buscar));
            }
            Err(error) => mostrar_error(&estado_buscar, error),
        }
    });

    if !updates.is_empty() {
        let grupo_actualizaciones = adw::PreferencesGroup::new();
        grupo_actualizaciones.set_title(&localizar_visible(
            idioma_actual(),
            "Actualizaciones disponibles",
        ));

        for update in updates {
            let id = update
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            if id.is_empty() {
                continue;
            }

            let nombre = update
                .get("name")
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty())
                .unwrap_or("Firmware")
                .to_string();

            let actual = update
                .get("currentVersion")
                .and_then(Value::as_str)
                .unwrap_or("—");
            let objetivo = update
                .pointer("/releases/0/version")
                .and_then(Value::as_str)
                .unwrap_or("—");

            let row = adw::ActionRow::new();
            row.set_title(&nombre);
            row.set_subtitle(&format!("{actual} → {objetivo}"));

            let boton = gtk::Button::with_label(texto(estado.idioma, "install"));
            boton.add_css_class("suggested-action");
            boton.set_valign(gtk::Align::Center);
            row.add_suffix(&boton);
            grupo_actualizaciones.add(&row);

            let estado_firmware = Rc::clone(&estado);
            let id_firmware = id.clone();
            let nombre_firmware = nombre.clone();

            boton.connect_clicked(move |boton| {
                let plan = match ejecutar_json(
                    &estado_firmware,
                    &["firmware", "update", &id_firmware, "--plan", "--json"],
                ) {
                    Ok(plan) => plan,
                    Err(error) => {
                        mostrar_error(&estado_firmware, error);
                        return;
                    }
                };

                let efecto = plan
                    .get("effect")
                    .and_then(Value::as_str)
                    .unwrap_or("immediate");

                let cuerpo =
                    frase_confirmar_firmware(estado_firmware.idioma, &nombre_firmware, efecto);
                let dialogo = dialogo_confirmacion(
                    boton,
                    estado_firmware.idioma,
                    &cuerpo,
                    texto(estado_firmware.idioma, "install"),
                    false,
                );

                let estado_aplicar = Rc::clone(&estado_firmware);
                let id_aplicar = id_firmware.clone();

                dialogo.connect_response(None, move |_, respuesta| {
                    if respuesta != "apply" {
                        return;
                    }

                    match ejecutar_json(
                        &estado_aplicar,
                        &["firmware", "update", &id_aplicar, "--yes", "--json"],
                    ) {
                        Ok(_) => {
                            mostrar_exito(
                                &estado_aplicar,
                                texto(estado_aplicar.idioma, "operation_done"),
                            );
                            recargar(Rc::clone(&estado_aplicar));
                        }
                        Err(error) => mostrar_error(&estado_aplicar, error),
                    }
                });

                dialogo.present();
            });
        }

        pagina.add(&grupo_actualizaciones);
    }

    if !problemas.is_empty() {
        let grupo_problemas = adw::PreferencesGroup::new();
        grupo_problemas.set_title(&localizar_visible(
            idioma_actual(),
            "Dispositivos que requieren atención",
        ));
        grupo_problemas.set_description(Some(&localizar_visible(
            idioma_actual(),
            "Se muestran solo dispositivos para los que fwupd informó un problema.",
        )));

        for device in problemas {
            let nombre = device
                .get("name")
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty() && *valor != "—")
                .or_else(|| {
                    device
                        .get("summary")
                        .and_then(Value::as_str)
                        .filter(|valor| !valor.trim().is_empty())
                })
                .or_else(|| {
                    device
                        .get("vendor")
                        .and_then(Value::as_str)
                        .filter(|valor| !valor.trim().is_empty())
                })
                .unwrap_or("Dispositivo de firmware");

            grupo_problemas.add(&fila(
                nombre,
                "fwupd informó que este dispositivo necesita revisión.",
            ));
        }

        pagina.add(&grupo_problemas);
    }

    pagina
}

fn texto_prueba_multimedia(idioma: Idioma, clave: &str) -> &'static str {
    match (idioma, clave) {
        (Idioma::Ingles, "output_test") => "Sound output test",
        (Idioma::BelarusLatino, "output_test") => "Test vychadu huku",
        (Idioma::Belarus, "output_test") => "Тэст выхаду гуку",
        (Idioma::Catalan, "output_test") => "Prova de sortida de so",
        (Idioma::Checo, "output_test") => "Test zvukového výstupu",
        (Idioma::Aleman, "output_test") => "Tonausgabetest",
        (Idioma::Frances, "output_test") => "Test de sortie sonore",
        (Idioma::Gallego, "output_test") => "Proba de saída de son",
        (Idioma::Italiano, "output_test") => "Prova di uscita del suono",
        (Idioma::Coreano, "output_test") => "사운드 출력 테스트",
        (Idioma::Kurdo, "output_test") => "Testa hilberîna deng",
        (Idioma::Neerlandes, "output_test") => "Geluidsuitgangstest",
        (Idioma::NoruegoNynorsk, "output_test") => "Test av lydutgang",
        (Idioma::Polaco, "output_test") => "Test wyjścia dźwięku",
        (Idioma::PortuguesBrasil, "output_test") => "Teste de saída de som",
        (Idioma::Ruso, "output_test") => "Тест вывода звука",
        (Idioma::Sueco, "output_test") => "Ljudutgångstest",
        (Idioma::Turco, "output_test") => "Ses çıkışı testi",
        (Idioma::Ucraniano, "output_test") => "Тест вихідного звуку",
        (Idioma::Vietnamita, "output_test") => "Kiểm tra đầu ra âm thanh",
        (Idioma::ChinoSimplificado, "output_test") => "声音输出测试",
        (Idioma::Ingles, "output_pick") => "Output to test",
        (Idioma::BelarusLatino, "output_pick") => "Vyvad dłja pravjerki",
        (Idioma::Belarus, "output_pick") => "Вывад для праверкі",
        (Idioma::Catalan, "output_pick") => "Sortida a provar",
        (Idioma::Checo, "output_pick") => "Výstup k testování",
        (Idioma::Aleman, "output_pick") => "Ausgabe zum Testen",
        (Idioma::Frances, "output_pick") => "Sortie à tester",
        (Idioma::Gallego, "output_pick") => "Saída para probar",
        (Idioma::Italiano, "output_pick") => "Uscita da testare",
        (Idioma::Coreano, "output_pick") => "테스트할 출력",
        (Idioma::Kurdo, "output_pick") => "Derketin ji bo ceribandinê",
        (Idioma::Neerlandes, "output_pick") => "Uitgang om te testen",
        (Idioma::NoruegoNynorsk, "output_pick") => "Utgang til test",
        (Idioma::Polaco, "output_pick") => "Dane wyjściowe do przetestowania",
        (Idioma::PortuguesBrasil, "output_pick") => "Saída para testar",
        (Idioma::Ruso, "output_pick") => "Выход для проверки",
        (Idioma::Sueco, "output_pick") => "Utgång att testa",
        (Idioma::Turco, "output_pick") => "Test edilecek çıktı",
        (Idioma::Ucraniano, "output_pick") => "Вихід для перевірки",
        (Idioma::Vietnamita, "output_pick") => "Đầu ra để kiểm tra",
        (Idioma::ChinoSimplificado, "output_pick") => "输出进行测试",
        (Idioma::Ingles, "output_detail") => {
            "Plays a short signal only through the selected output. It does not change the default output."
        }
        (Idioma::BelarusLatino, "output_detail") => {
            "Prajhraje karotki sihnał tołki praz vybrany vychad. Heta nje zmjanjaje vyvad pa zmaŭčanni."
        }
        (Idioma::Belarus, "output_detail") => {
            "Прайграе кароткі сігнал толькі праз выбраны выхад. Гэта не змяняе вывад па змаўчанні."
        }
        (Idioma::Catalan, "output_detail") => {
            "Reprodueix un senyal curt només a través de la sortida seleccionada. No canvia la sortida predeterminada."
        }
        (Idioma::Checo, "output_detail") => {
            "Přehraje krátký signál pouze přes vybraný výstup. Nemění výchozí výstup."
        }
        (Idioma::Aleman, "output_detail") => {
            "Spielt ein kurzes Signal nur über den ausgewählten Ausgang ab. Die Standardausgabe wird dadurch nicht geändert."
        }
        (Idioma::Frances, "output_detail") => {
            "Lit un signal court uniquement via la sortie sélectionnée. Cela ne modifie pas la sortie par défaut."
        }
        (Idioma::Gallego, "output_detail") => {
            "Reproduce un sinal curto só a través da saída seleccionada. Non cambia a saída predeterminada."
        }
        (Idioma::Italiano, "output_detail") => {
            "Riproduce un segnale breve solo attraverso l'uscita selezionata. Non modifica l'output predefinito."
        }
        (Idioma::Coreano, "output_detail") => {
            "선택한 출력을 통해서만 짧은 신호를 재생합니다. 기본 출력은 변경되지 않습니다."
        }
        (Idioma::Kurdo, "output_detail") => {
            "Tenê di nav derana hilbijartî de îşaretek kurt dilîze. Ew hilberîna xwerû naguhere."
        }
        (Idioma::Neerlandes, "output_detail") => {
            "Speelt alleen een kort signaal af via de geselecteerde uitgang. De standaarduitvoer wordt niet gewijzigd."
        }
        (Idioma::NoruegoNynorsk, "output_detail") => {
            "Spiller et kort signal kun gjennom den valgte utgangen. Det endrer ikke standardutgangen."
        }
        (Idioma::Polaco, "output_detail") => {
            "Odtwarza krótki sygnał tylko przez wybrane wyjście. Nie zmienia to domyślnego wyjścia."
        }
        (Idioma::PortuguesBrasil, "output_detail") => {
            "Reproduz um sinal curto somente através da saída selecionada. Isso não altera a saída padrão."
        }
        (Idioma::Ruso, "output_detail") => {
            "Воспроизводит короткий сигнал только через выбранный выход. Это не меняет вывод по умолчанию."
        }
        (Idioma::Sueco, "output_detail") => {
            "Spelar en kort signal endast genom den valda utgången. Det ändrar inte standardutgången."
        }
        (Idioma::Turco, "output_detail") => {
            "Yalnızca seçilen çıkış yoluyla kısa bir sinyal oynatır. Varsayılan çıktıyı değiştirmez."
        }
        (Idioma::Ucraniano, "output_detail") => {
            "Відтворює короткий сигнал лише через вибраний вихід. Це не змінює стандартний вихід."
        }
        (Idioma::Vietnamita, "output_detail") => {
            "Chỉ phát tín hiệu ngắn thông qua đầu ra đã chọn. Nó không thay đổi đầu ra mặc định."
        }
        (Idioma::ChinoSimplificado, "output_detail") => {
            "仅通过选定的输出播放短信号。它不会更改默认输出。"
        }
        (Idioma::Ingles, "both") => "Test sound",
        (Idioma::BelarusLatino, "both") => "Test huku",
        (Idioma::Belarus, "both") => "Тэст гуку",
        (Idioma::Catalan, "both") => "Prova de so",
        (Idioma::Checo, "both") => "Testovací zvuk",
        (Idioma::Aleman, "both") => "Ton testen",
        (Idioma::Frances, "both") => "Tester le son",
        (Idioma::Gallego, "both") => "Proba de son",
        (Idioma::Italiano, "both") => "Prova il suono",
        (Idioma::Coreano, "both") => "테스트 사운드",
        (Idioma::Kurdo, "both") => "Dengê testê",
        (Idioma::Neerlandes, "both") => "Geluid testen",
        (Idioma::NoruegoNynorsk, "both") => "Test lyd",
        (Idioma::Polaco, "both") => "Testuj dźwięk",
        (Idioma::PortuguesBrasil, "both") => "Teste de som",
        (Idioma::Ruso, "both") => "Тестовый звук",
        (Idioma::Sueco, "both") => "Testljud",
        (Idioma::Turco, "both") => "Sesi test et",
        (Idioma::Ucraniano, "both") => "Тестовий звук",
        (Idioma::Vietnamita, "both") => "Kiểm tra âm thanh",
        (Idioma::ChinoSimplificado, "both") => "测试声音",
        (Idioma::Ingles, "left") => "Left",
        (Idioma::BelarusLatino, "left") => "nałjeva",
        (Idioma::Belarus, "left") => "Злева",
        (Idioma::Catalan, "left") => "Esquerra",
        (Idioma::Checo, "left") => "Vlevo",
        (Idioma::Aleman, "left") => "Links",
        (Idioma::Frances, "left") => "Gauche",
        (Idioma::Gallego, "left") => "Esquerda",
        (Idioma::Italiano, "left") => "Sinistra",
        (Idioma::Coreano, "left") => "좌측",
        (Idioma::Kurdo, "left") => "Çep",
        (Idioma::Neerlandes, "left") => "Links",
        (Idioma::NoruegoNynorsk, "left") => "Venstre",
        (Idioma::Polaco, "left") => "Lewo",
        (Idioma::PortuguesBrasil, "left") => "Esquerda",
        (Idioma::Ruso, "left") => "Слева",
        (Idioma::Sueco, "left") => "Vänster",
        (Idioma::Turco, "left") => "Sol",
        (Idioma::Ucraniano, "left") => "Ліворуч",
        (Idioma::Vietnamita, "left") => "Trái",
        (Idioma::ChinoSimplificado, "left") => "左侧",
        (Idioma::Ingles, "right") => "Right",
        (Idioma::BelarusLatino, "right") => "praviłna",
        (Idioma::Belarus, "right") => "Справа",
        (Idioma::Catalan, "right") => "Dreta",
        (Idioma::Checo, "right") => "Vpravo",
        (Idioma::Aleman, "right") => "Rechts",
        (Idioma::Frances, "right") => "Droite",
        (Idioma::Gallego, "right") => "Dereita",
        (Idioma::Italiano, "right") => "Destra",
        (Idioma::Coreano, "right") => "우측",
        (Idioma::Kurdo, "right") => "Rast",
        (Idioma::Neerlandes, "right") => "Rechts",
        (Idioma::NoruegoNynorsk, "right") => "Høgre",
        (Idioma::Polaco, "right") => "Prawo",
        (Idioma::PortuguesBrasil, "right") => "Direita",
        (Idioma::Ruso, "right") => "Справа",
        (Idioma::Sueco, "right") => "Rätt",
        (Idioma::Turco, "right") => "Sağ",
        (Idioma::Ucraniano, "right") => "Праворуч",
        (Idioma::Vietnamita, "right") => "Phải",
        (Idioma::ChinoSimplificado, "right") => "右侧",
        (Idioma::Ingles, "output_done") => "Sound output test completed.",
        (Idioma::BelarusLatino, "output_done") => "Test vychadu huku zavjeršany.",
        (Idioma::Belarus, "output_done") => "Тэст выхаду гуку завершаны.",
        (Idioma::Catalan, "output_done") => "S'ha completat la prova de sortida del so.",
        (Idioma::Checo, "output_done") => "Test zvukového výstupu dokončen.",
        (Idioma::Aleman, "output_done") => "Tonausgabetest abgeschlossen.",
        (Idioma::Frances, "output_done") => "Test de sortie sonore terminé.",
        (Idioma::Gallego, "output_done") => "Proba de saída de son rematada.",
        (Idioma::Italiano, "output_done") => "Test dell'uscita audio completato.",
        (Idioma::Coreano, "output_done") => "사운드 출력 테스트가 완료되었습니다.",
        (Idioma::Kurdo, "output_done") => "Testa derketina deng qediya.",
        (Idioma::Neerlandes, "output_done") => "Test geluidsuitvoer voltooid.",
        (Idioma::NoruegoNynorsk, "output_done") => "Lydutgangstest fullført.",
        (Idioma::Polaco, "output_done") => "Zakończono test wyjścia dźwięku.",
        (Idioma::PortuguesBrasil, "output_done") => "Teste de saída de som concluído.",
        (Idioma::Ruso, "output_done") => "Проверка вывода звука завершена.",
        (Idioma::Sueco, "output_done") => "Ljudutgångstest avslutat.",
        (Idioma::Turco, "output_done") => "Ses çıkışı testi tamamlandı.",
        (Idioma::Ucraniano, "output_done") => "Перевірку вихідного звуку завершено.",
        (Idioma::Vietnamita, "output_done") => "Kiểm tra đầu ra âm thanh đã hoàn thành.",
        (Idioma::ChinoSimplificado, "output_done") => "声音输出测试完成。",
        (Idioma::Ingles, "mic_test") => "Test microphone",
        (Idioma::BelarusLatino, "mic_test") => "Test mikrafona",
        (Idioma::Belarus, "mic_test") => "Тэст мікрафона",
        (Idioma::Catalan, "mic_test") => "Micròfon de prova",
        (Idioma::Checo, "mic_test") => "Testovací mikrofon",
        (Idioma::Aleman, "mic_test") => "Testmikrofon",
        (Idioma::Frances, "mic_test") => "Tester le microphone",
        (Idioma::Gallego, "mic_test") => "Micrófono de proba",
        (Idioma::Italiano, "mic_test") => "Prova il microfono",
        (Idioma::Coreano, "mic_test") => "마이크 테스트",
        (Idioma::Kurdo, "mic_test") => "Mîkrofona testê",
        (Idioma::Neerlandes, "mic_test") => "Microfoon testen",
        (Idioma::NoruegoNynorsk, "mic_test") => "Test mikrofon",
        (Idioma::Polaco, "mic_test") => "Mikrofon testowy",
        (Idioma::PortuguesBrasil, "mic_test") => "Microfone de teste",
        (Idioma::Ruso, "mic_test") => "Тестовый микрофон",
        (Idioma::Sueco, "mic_test") => "Testa mikrofonen",
        (Idioma::Turco, "mic_test") => "Mikrofonu test edin",
        (Idioma::Ucraniano, "mic_test") => "Тестовий мікрофон",
        (Idioma::Vietnamita, "mic_test") => "Kiểm tra micrô",
        (Idioma::ChinoSimplificado, "mic_test") => "测试麦克风",
        (Idioma::Ingles, "mic_pick") => "Microphone to test",
        (Idioma::BelarusLatino, "mic_pick") => "Mikrafon dłja pravjerki",
        (Idioma::Belarus, "mic_pick") => "Мікрафон для праверкі",
        (Idioma::Catalan, "mic_pick") => "Micròfon per provar",
        (Idioma::Checo, "mic_pick") => "Mikrofon na zkoušku",
        (Idioma::Aleman, "mic_pick") => "Mikrofon zum Testen",
        (Idioma::Frances, "mic_pick") => "Micro à tester",
        (Idioma::Gallego, "mic_pick") => "Micrófono para probar",
        (Idioma::Italiano, "mic_pick") => "Microfono da testare",
        (Idioma::Coreano, "mic_pick") => "테스트할 마이크",
        (Idioma::Kurdo, "mic_pick") => "Mîkrofon ji bo ceribandinê",
        (Idioma::Neerlandes, "mic_pick") => "Microfoon om te testen",
        (Idioma::NoruegoNynorsk, "mic_pick") => "Mikrofon for å teste",
        (Idioma::Polaco, "mic_pick") => "Mikrofon do przetestowania",
        (Idioma::PortuguesBrasil, "mic_pick") => "Microfone para testar",
        (Idioma::Ruso, "mic_pick") => "Микрофон для проверки",
        (Idioma::Sueco, "mic_pick") => "Mikrofon att testa",
        (Idioma::Turco, "mic_pick") => "Test edilecek mikrofon",
        (Idioma::Ucraniano, "mic_pick") => "Мікрофон для перевірки",
        (Idioma::Vietnamita, "mic_pick") => "Micrô để kiểm tra",
        (Idioma::ChinoSimplificado, "mic_pick") => "待测试麦克风",
        (Idioma::Ingles, "mic_level") => "Live microphone level",
        (Idioma::BelarusLatino, "mic_level") => "Žyvy ŭzrovjen mikrafona",
        (Idioma::Belarus, "mic_level") => "Жывы ўзровень мікрафона",
        (Idioma::Catalan, "mic_level") => "Nivell de micròfon en directe",
        (Idioma::Checo, "mic_level") => "Živá úroveň mikrofonu",
        (Idioma::Aleman, "mic_level") => "Live-Mikrofonpegel",
        (Idioma::Frances, "mic_level") => "Niveau du microphone en direct",
        (Idioma::Gallego, "mic_level") => "Nivel de micrófono en directo",
        (Idioma::Italiano, "mic_level") => "Livello del microfono dal vivo",
        (Idioma::Coreano, "mic_level") => "라이브 마이크 레벨",
        (Idioma::Kurdo, "mic_level") => "Asta mîkrofona zindî",
        (Idioma::Neerlandes, "mic_level") => "Live microfoonniveau",
        (Idioma::NoruegoNynorsk, "mic_level") => "Live mikrofonnivå",
        (Idioma::Polaco, "mic_level") => "Poziom mikrofonu na żywo",
        (Idioma::PortuguesBrasil, "mic_level") => "Nível do microfone ao vivo",
        (Idioma::Ruso, "mic_level") => "Уровень живого микрофона",
        (Idioma::Sueco, "mic_level") => "Live mikrofonnivå",
        (Idioma::Turco, "mic_level") => "Canlı mikrofon seviyesi",
        (Idioma::Ucraniano, "mic_level") => "Живий рівень мікрофона",
        (Idioma::Vietnamita, "mic_level") => "Mức micrô trực tiếp",
        (Idioma::ChinoSimplificado, "mic_level") => "现场麦克风电平",
        (Idioma::Ingles, "mic_level_detail") => {
            "Speak normally while the meter is active. Press Stop measuring when you are done."
        }
        (Idioma::BelarusLatino, "mic_level_detail") => {
            "Razmaŭłjajcje zvyčajna, pakuł łičyłnik aktyŭny. Nacisnicje Spynic vymjarennje, kałi skončycje."
        }
        (Idioma::Belarus, "mic_level_detail") => {
            "Размаўляйце звычайна, пакуль лічыльнік актыўны. Націсніце Спыніць вымярэнне, калі скончыце."
        }
        (Idioma::Catalan, "mic_level_detail") => {
            "Parleu normalment mentre el mesurador està actiu. Premeu Atura de mesurar quan hàgiu acabat."
        }
        (Idioma::Checo, "mic_level_detail") => {
            "Když je měřič aktivní, mluvte normálně. Až budete hotovi, stiskněte Zastavit měření."
        }
        (Idioma::Aleman, "mic_level_detail") => {
            "Sprechen Sie normal, während das Messgerät aktiv ist. Drücken Sie „Messung stoppen“, wenn Sie fertig sind."
        }
        (Idioma::Frances, "mic_level_detail") => {
            "Parlez normalement lorsque le compteur est actif. Appuyez sur Arrêter de mesurer lorsque vous avez terminé."
        }
        (Idioma::Gallego, "mic_level_detail") => {
            "Fala normalmente mentres o medidor está activo. Preme Deixar de medir cando remates."
        }
        (Idioma::Italiano, "mic_level_detail") => {
            "Parla normalmente mentre lo strumento è attivo. Premere Interrompi misurazione quando hai finito."
        }
        (Idioma::Coreano, "mic_level_detail") => {
            "미터가 활성화된 동안 정상적으로 말하십시오. 완료되면 측정 중지를 누르세요."
        }
        (Idioma::Kurdo, "mic_level_detail") => {
            "Dema ku metre çalak e bi gelemperî biaxivin. Dema ku hûn qediyan, pêl Stop pîvandinê bikin."
        }
        (Idioma::Neerlandes, "mic_level_detail") => {
            "Spreek normaal terwijl de meter actief is. Druk op Stop met meten als u klaar bent."
        }
        (Idioma::NoruegoNynorsk, "mic_level_detail") => {
            "Snakk normalt mens måleren er aktiv. Trykk Stopp måling når du er ferdig."
        }
        (Idioma::Polaco, "mic_level_detail") => {
            "Mów normalnie, gdy licznik jest aktywny. Po zakończeniu naciśnij przycisk Zatrzymaj pomiar."
        }
        (Idioma::PortuguesBrasil, "mic_level_detail") => {
            "Fale normalmente enquanto o medidor estiver ativo. Pressione Parar medição quando terminar."
        }
        (Idioma::Ruso, "mic_level_detail") => {
            "Говорите нормально, пока глюкометр активен. Когда закончите, нажмите «Остановить измерение»."
        }
        (Idioma::Sueco, "mic_level_detail") => {
            "Tala normalt medan mätaren är aktiv. Tryck på Stoppa mätning när du är klar."
        }
        (Idioma::Turco, "mic_level_detail") => {
            "Sayaç aktifken normal konuşun. İşiniz bittiğinde Ölçümü durdur'a basın."
        }
        (Idioma::Ucraniano, "mic_level_detail") => {
            "Говоріть нормально, поки лічильник активний. Натисніть Зупинити вимірювання, коли закінчите."
        }
        (Idioma::Vietnamita, "mic_level_detail") => {
            "Nói chuyện bình thường trong khi đồng hồ đang hoạt động. Nhấn Dừng đo khi bạn hoàn tất."
        }
        (Idioma::ChinoSimplificado, "mic_level_detail") => {
            "当仪表处于活动状态时正常说话。完成后按停止测量。"
        }
        (Idioma::Ingles, "measure") => "Start measuring",
        (Idioma::BelarusLatino, "measure") => "Pačnicje vymjarac",
        (Idioma::Belarus, "measure") => "Пачніце вымяраць",
        (Idioma::Catalan, "measure") => "Comença a mesurar",
        (Idioma::Checo, "measure") => "Začněte měřit",
        (Idioma::Aleman, "measure") => "Beginnen Sie mit der Messung",
        (Idioma::Frances, "measure") => "Commencez à mesurer",
        (Idioma::Gallego, "measure") => "Comeza a medir",
        (Idioma::Italiano, "measure") => "Inizia a misurare",
        (Idioma::Coreano, "measure") => "측정 시작",
        (Idioma::Kurdo, "measure") => "Dest bi pîvandinê bikin",
        (Idioma::Neerlandes, "measure") => "Begin met meten",
        (Idioma::NoruegoNynorsk, "measure") => "Begynn å måle",
        (Idioma::Polaco, "measure") => "Rozpocznij pomiar",
        (Idioma::PortuguesBrasil, "measure") => "Comece a medir",
        (Idioma::Ruso, "measure") => "Начать измерение",
        (Idioma::Sueco, "measure") => "Börja mäta",
        (Idioma::Turco, "measure") => "Ölçüme başlayın",
        (Idioma::Ucraniano, "measure") => "Почніть вимірювати",
        (Idioma::Vietnamita, "measure") => "Bắt đầu đo",
        (Idioma::ChinoSimplificado, "measure") => "开始测量",
        (Idioma::Ingles, "measure_stop") => "Stop measuring",
        (Idioma::BelarusLatino, "measure_stop") => "Spynic vymjarennje",
        (Idioma::Belarus, "measure_stop") => "Спыніць вымярэнне",
        (Idioma::Catalan, "measure_stop") => "Deixa de mesurar",
        (Idioma::Checo, "measure_stop") => "Přestaňte měřit",
        (Idioma::Aleman, "measure_stop") => "Hören Sie auf zu messen",
        (Idioma::Frances, "measure_stop") => "Arrêter de mesurer",
        (Idioma::Gallego, "measure_stop") => "Deixa de medir",
        (Idioma::Italiano, "measure_stop") => "Smetti di misurare",
        (Idioma::Coreano, "measure_stop") => "측정 중지",
        (Idioma::Kurdo, "measure_stop") => "Pîvan rawestînin",
        (Idioma::Neerlandes, "measure_stop") => "Stop met meten",
        (Idioma::NoruegoNynorsk, "measure_stop") => "Slutt å måle",
        (Idioma::Polaco, "measure_stop") => "Przestań mierzyć",
        (Idioma::PortuguesBrasil, "measure_stop") => "Pare de medir",
        (Idioma::Ruso, "measure_stop") => "Прекратить измерение",
        (Idioma::Sueco, "measure_stop") => "Sluta mäta",
        (Idioma::Turco, "measure_stop") => "Ölçümü durdur",
        (Idioma::Ucraniano, "measure_stop") => "Припинити вимірювання",
        (Idioma::Vietnamita, "measure_stop") => "Dừng đo",
        (Idioma::ChinoSimplificado, "measure_stop") => "停止测量",
        (Idioma::Ingles, "record") => "Record test",
        (Idioma::BelarusLatino, "record") => "Zapis testu",
        (Idioma::Belarus, "record") => "Запіс тэсту",
        (Idioma::Catalan, "record") => "Prova de registre",
        (Idioma::Checo, "record") => "Záznam testu",
        (Idioma::Aleman, "record") => "Rekordtest",
        (Idioma::Frances, "record") => "Test d'enregistrement",
        (Idioma::Gallego, "record") => "Proba de rexistro",
        (Idioma::Italiano, "record") => "Prova di registrazione",
        (Idioma::Coreano, "record") => "기록 테스트",
        (Idioma::Kurdo, "record") => "Testê tomar bikin",
        (Idioma::Neerlandes, "record") => "Test opnemen",
        (Idioma::NoruegoNynorsk, "record") => "Rekordprøve",
        (Idioma::Polaco, "record") => "Rekordowy test",
        (Idioma::PortuguesBrasil, "record") => "Teste de registro",
        (Idioma::Ruso, "record") => "Запись теста",
        (Idioma::Sueco, "record") => "Rekordtest",
        (Idioma::Turco, "record") => "Testi kaydet",
        (Idioma::Ucraniano, "record") => "Запис тесту",
        (Idioma::Vietnamita, "record") => "Ghi lại bài kiểm tra",
        (Idioma::ChinoSimplificado, "record") => "记录测试",
        (Idioma::Ingles, "record_again") => "Record again",
        (Idioma::BelarusLatino, "record_again") => "Zapis jašče raz",
        (Idioma::Belarus, "record_again") => "Запіс яшчэ раз",
        (Idioma::Catalan, "record_again") => "Torna a gravar",
        (Idioma::Checo, "record_again") => "Nahrajte znovu",
        (Idioma::Aleman, "record_again") => "Nochmals aufnehmen",
        (Idioma::Frances, "record_again") => "Enregistrer à nouveau",
        (Idioma::Gallego, "record_again") => "Gravar de novo",
        (Idioma::Italiano, "record_again") => "Registra di nuovo",
        (Idioma::Coreano, "record_again") => "다시 녹음하세요",
        (Idioma::Kurdo, "record_again") => "Dîsa tomar bikin",
        (Idioma::Neerlandes, "record_again") => "Neem opnieuw op",
        (Idioma::NoruegoNynorsk, "record_again") => "Ta opp igjen",
        (Idioma::Polaco, "record_again") => "Nagraj ponownie",
        (Idioma::PortuguesBrasil, "record_again") => "Grave novamente",
        (Idioma::Ruso, "record_again") => "Записать еще раз",
        (Idioma::Sueco, "record_again") => "Spela in igen",
        (Idioma::Turco, "record_again") => "Tekrar kaydet",
        (Idioma::Ucraniano, "record_again") => "Запис знову",
        (Idioma::Vietnamita, "record_again") => "Ghi lại",
        (Idioma::ChinoSimplificado, "record_again") => "再次录制",
        (Idioma::Ingles, "recording_button") => "Recording…",
        (Idioma::BelarusLatino, "recording_button") => "Zapis...",
        (Idioma::Belarus, "recording_button") => "Запіс...",
        (Idioma::Catalan, "recording_button") => "Enregistrament...",
        (Idioma::Checo, "recording_button") => "Záznam…",
        (Idioma::Aleman, "recording_button") => "Aufnahme…",
        (Idioma::Frances, "recording_button") => "Enregistrement…",
        (Idioma::Gallego, "recording_button") => "Gravando…",
        (Idioma::Italiano, "recording_button") => "Registrazione…",
        (Idioma::Coreano, "recording_button") => "녹음…",
        (Idioma::Kurdo, "recording_button") => "Girtinî…",
        (Idioma::Neerlandes, "recording_button") => "Opnemen…",
        (Idioma::NoruegoNynorsk, "recording_button") => "Innspilling…",
        (Idioma::Polaco, "recording_button") => "Nagranie…",
        (Idioma::PortuguesBrasil, "recording_button") => "Gravação…",
        (Idioma::Ruso, "recording_button") => "Запись…",
        (Idioma::Sueco, "recording_button") => "Inspelning…",
        (Idioma::Turco, "recording_button") => "Kayıt…",
        (Idioma::Ucraniano, "recording_button") => "Запис...",
        (Idioma::Vietnamita, "recording_button") => "Đang ghi…",
        (Idioma::ChinoSimplificado, "recording_button") => "记录…",
        (Idioma::Ingles, "playing_button") => "Playing…",
        (Idioma::BelarusLatino, "playing_button") => "Prajhravannje...",
        (Idioma::Belarus, "playing_button") => "Прайграванне...",
        (Idioma::Catalan, "playing_button") => "S'està jugant…",
        (Idioma::Checo, "playing_button") => "Přehrává se…",
        (Idioma::Aleman, "playing_button") => "Spielen…",
        (Idioma::Frances, "playing_button") => "Jouant…",
        (Idioma::Gallego, "playing_button") => "Xogando…",
        (Idioma::Italiano, "playing_button") => "Giocando…",
        (Idioma::Coreano, "playing_button") => "재생 중…",
        (Idioma::Kurdo, "playing_button") => "Dilîstin…",
        (Idioma::Neerlandes, "playing_button") => "Spelen…",
        (Idioma::NoruegoNynorsk, "playing_button") => "Spiller …",
        (Idioma::Polaco, "playing_button") => "Gra…",
        (Idioma::PortuguesBrasil, "playing_button") => "Jogando…",
        (Idioma::Ruso, "playing_button") => "Игра…",
        (Idioma::Sueco, "playing_button") => "Spelar...",
        (Idioma::Turco, "playing_button") => "Oynanıyor…",
        (Idioma::Ucraniano, "playing_button") => "Відтворення…",
        (Idioma::Vietnamita, "playing_button") => "Đang chơi…",
        (Idioma::ChinoSimplificado, "playing_button") => "正在播放…",
        (Idioma::Ingles, "play") => "Play test",
        (Idioma::BelarusLatino, "play") => "Hułjac u test",
        (Idioma::Belarus, "play") => "Гуляць у тэст",
        (Idioma::Catalan, "play") => "Prova de joc",
        (Idioma::Checo, "play") => "Přehrát test",
        (Idioma::Aleman, "play") => "Spieltest",
        (Idioma::Frances, "play") => "Jouer à l'essai",
        (Idioma::Gallego, "play") => "Proba de xogo",
        (Idioma::Italiano, "play") => "Gioca alla prova",
        (Idioma::Coreano, "play") => "테스트 플레이",
        (Idioma::Kurdo, "play") => "Test play",
        (Idioma::Neerlandes, "play") => "Speeltest",
        (Idioma::NoruegoNynorsk, "play") => "Spilleprøve",
        (Idioma::Polaco, "play") => "Zagraj w próbę",
        (Idioma::PortuguesBrasil, "play") => "Teste de jogo",
        (Idioma::Ruso, "play") => "Игровой тест",
        (Idioma::Sueco, "play") => "Spelprov",
        (Idioma::Turco, "play") => "Testi oyna",
        (Idioma::Ucraniano, "play") => "Грати в тест",
        (Idioma::Vietnamita, "play") => "Chơi thử",
        (Idioma::ChinoSimplificado, "play") => "播放测试",
        (Idioma::Ingles, "voice_detail") => {
            "Records a short private sample. Playback deletes the temporary recording automatically."
        }
        (Idioma::BelarusLatino, "voice_detail") => {
            "Zapisvaje karotki pryvatny ŭzor. Prajhravannje aŭtamatyčna vydałjaje časovy zapis."
        }
        (Idioma::Belarus, "voice_detail") => {
            "Запісвае кароткі прыватны ўзор. Прайграванне аўтаматычна выдаляе часовы запіс."
        }
        (Idioma::Catalan, "voice_detail") => {
            "Grava una breu mostra privada. La reproducció elimina automàticament l'enregistrament temporal."
        }
        (Idioma::Checo, "voice_detail") => {
            "Nahrává krátkou soukromou ukázku. Přehrávání automaticky smaže dočasný záznam."
        }
        (Idioma::Aleman, "voice_detail") => {
            "Nimmt eine kurze private Probe auf. Bei der Wiedergabe wird die temporäre Aufnahme automatisch gelöscht."
        }
        (Idioma::Frances, "voice_detail") => {
            "Enregistre un court échantillon privé. La lecture supprime automatiquement l’enregistrement temporaire."
        }
        (Idioma::Gallego, "voice_detail") => {
            "Grava unha pequena mostra privada. A reprodución elimina automaticamente a gravación temporal."
        }
        (Idioma::Italiano, "voice_detail") => {
            "Registra un breve campione privato. La riproduzione elimina automaticamente la registrazione temporanea."
        }
        (Idioma::Coreano, "voice_detail") => {
            "짧은 개인 샘플을 녹음합니다. 재생하면 임시 녹음이 자동으로 삭제됩니다."
        }
        (Idioma::Kurdo, "voice_detail") => {
            "Nimûneyek taybet a kurt tomar dike. Playback bixweber tomarkirina demkî jêdibe."
        }
        (Idioma::Neerlandes, "voice_detail") => {
            "Neemt een korte privésample op. Bij het afspelen wordt de tijdelijke opname automatisch verwijderd."
        }
        (Idioma::NoruegoNynorsk, "voice_detail") => {
            "Tar opp en kort privat prøve. Avspilling sletter det midlertidige opptaket automatisk."
        }
        (Idioma::Polaco, "voice_detail") => {
            "Rejestruje krótką prywatną próbkę. Odtwarzanie automatycznie usuwa nagranie tymczasowe."
        }
        (Idioma::PortuguesBrasil, "voice_detail") => {
            "Grava uma pequena amostra privada. A reprodução exclui a gravação temporária automaticamente."
        }
        (Idioma::Ruso, "voice_detail") => {
            "Записывает короткий частный образец. При воспроизведении временная запись автоматически удаляется."
        }
        (Idioma::Sueco, "voice_detail") => {
            "Spelar in ett kort privat prov. Uppspelning raderar den tillfälliga inspelningen automatiskt."
        }
        (Idioma::Turco, "voice_detail") => {
            "Kısa bir özel örnek kaydeder. Oynatma, geçici kaydı otomatik olarak siler."
        }
        (Idioma::Ucraniano, "voice_detail") => {
            "Записує короткий приватний зразок. Відтворення автоматично видаляє тимчасовий запис."
        }
        (Idioma::Vietnamita, "voice_detail") => {
            "Ghi lại một mẫu riêng tư ngắn. Phát lại sẽ tự động xóa bản ghi tạm thời."
        }
        (Idioma::ChinoSimplificado, "voice_detail") => {
            "记录一个简短的私人样本。播放时会自动删除临时录音。"
        }
        (Idioma::Ingles, "recorded") => "Temporary microphone sample recorded.",
        (Idioma::BelarusLatino, "recorded") => "Zapisany ŭzor časovaha mikrafona.",
        (Idioma::Belarus, "recorded") => "Запісаны ўзор часовага мікрафона.",
        (Idioma::Catalan, "recorded") => "S'ha gravat una mostra de micròfon temporal.",
        (Idioma::Checo, "recorded") => "Dočasný záznam mikrofonu.",
        (Idioma::Aleman, "recorded") => "Temporäres Mikrofon-Sample aufgenommen.",
        (Idioma::Frances, "recorded") => "Échantillon de microphone temporaire enregistré.",
        (Idioma::Gallego, "recorded") => "Mostra de micrófono temporal gravada.",
        (Idioma::Italiano, "recorded") => "Campione temporaneo registrato dal microfono.",
        (Idioma::Coreano, "recorded") => "임시 마이크 샘플이 녹음되었습니다.",
        (Idioma::Kurdo, "recorded") => "Nimûneya mîkrofona demkî hate tomar kirin.",
        (Idioma::Neerlandes, "recorded") => "Tijdelijk microfoonmonster opgenomen.",
        (Idioma::NoruegoNynorsk, "recorded") => "Midlertidig mikrofonprøve tatt opp.",
        (Idioma::Polaco, "recorded") => "Nagrano tymczasową próbkę mikrofonu.",
        (Idioma::PortuguesBrasil, "recorded") => "Amostra temporária de microfone gravada.",
        (Idioma::Ruso, "recorded") => "Записан временный образец микрофона.",
        (Idioma::Sueco, "recorded") => "Tillfälligt mikrofonprov inspelat.",
        (Idioma::Turco, "recorded") => "Geçici mikrofon örneği kaydedildi.",
        (Idioma::Ucraniano, "recorded") => "Тимчасовий зразок мікрофона записаний.",
        (Idioma::Vietnamita, "recorded") => "Đã ghi lại mẫu micrô tạm thời.",
        (Idioma::ChinoSimplificado, "recorded") => "录制临时麦克风样本。",
        (Idioma::Ingles, "played") => "Microphone sample played and deleted.",
        (Idioma::BelarusLatino, "played") => "Uzor mikrafona prajhrany i vydałjeny.",
        (Idioma::Belarus, "played") => "Узор мікрафона прайграны і выдалены.",
        (Idioma::Catalan, "played") => "S'ha reproduït i esborrat la mostra del micròfon.",
        (Idioma::Checo, "played") => "Vzorek mikrofonu přehrán a odstraněn.",
        (Idioma::Aleman, "played") => "Mikrofonbeispiel abgespielt und gelöscht.",
        (Idioma::Frances, "played") => "Échantillon de microphone joué et supprimé.",
        (Idioma::Gallego, "played") => "Mostra de micrófono reproducida e eliminada.",
        (Idioma::Italiano, "played") => "Campione del microfono riprodotto ed eliminato.",
        (Idioma::Coreano, "played") => "마이크 샘플이 재생되고 삭제되었습니다.",
        (Idioma::Kurdo, "played") => "Nimûneya mîkrofonê hate lîstin û jêbirin.",
        (Idioma::Neerlandes, "played") => "Microfoonvoorbeeld afgespeeld en verwijderd.",
        (Idioma::NoruegoNynorsk, "played") => "Mikrofoneksempel spilt av og slettet.",
        (Idioma::Polaco, "played") => "Próbka mikrofonu odtworzona i usunięta.",
        (Idioma::PortuguesBrasil, "played") => "Amostra de microfone reproduzida e excluída.",
        (Idioma::Ruso, "played") => "Сэмпл микрофона воспроизведен и удален.",
        (Idioma::Sueco, "played") => "Mikrofonexempel spelas upp och raderas.",
        (Idioma::Turco, "played") => "Mikrofon örneği oynatıldı ve silindi.",
        (Idioma::Ucraniano, "played") => "Зразок мікрофона відтворено та видалено.",
        (Idioma::Vietnamita, "played") => "Mẫu micrô đã phát và xóa.",
        (Idioma::ChinoSimplificado, "played") => "麦克风样本已播放并删除。",

        (Idioma::Hungaro, "output_test") => "Hangkimenet tesztje",
        (Idioma::Hungaro, "output_pick") => "Tesztelendő kimenet",
        (Idioma::Hungaro, "output_detail") => {
            "Rövid jelet játszik le csak a kiválasztott kimeneten. Nem módosítja az alapértelmezett kimenetet."
        }
        (Idioma::Hungaro, "both") => "Hang tesztelése",
        (Idioma::Hungaro, "left") => "Bal",
        (Idioma::Hungaro, "right") => "Jobb",
        (Idioma::Hungaro, "output_done") => "A hangkimenet tesztje befejeződött.",
        (Idioma::Hungaro, "mic_test") => "Mikrofon tesztelése",
        (Idioma::Hungaro, "mic_pick") => "Tesztelendő mikrofon",
        (Idioma::Hungaro, "mic_level") => "Élő mikrofonszint",
        (Idioma::Hungaro, "mic_level_detail") => {
            "Beszélj normál hangerővel, amíg a mérés aktív. Ha végeztél, állítsd le a mérést."
        }
        (Idioma::Hungaro, "measure") => "Mérés indítása",
        (Idioma::Hungaro, "measure_stop") => "Mérés leállítása",
        (Idioma::Hungaro, "record") => "Próba felvétele",
        (Idioma::Hungaro, "record_again") => "Új felvétel",
        (Idioma::Hungaro, "recording_button") => "Felvétel…",
        (Idioma::Hungaro, "playing_button") => "Lejátszás…",
        (Idioma::Hungaro, "play") => "Próba lejátszása",
        (Idioma::Hungaro, "voice_detail") => {
            "Rövid privát mintát rögzít. A lejátszás automatikusan törli az ideiglenes felvételt."
        }
        (Idioma::Hungaro, "recorded") => "Az ideiglenes mikrofonminta elkészült.",
        (Idioma::Hungaro, "played") => "A mikrofonminta lejátszva és törölve.",

        (_, "output_test") => "Prueba de salida de sonido",
        (_, "output_pick") => "Salida para la prueba",
        (_, "output_detail") => {
            "Reproduce una señal breve solo por la salida elegida. No cambia la salida predeterminada."
        }
        (_, "both") => "Probar sonido",
        (_, "left") => "Izquierda",
        (_, "right") => "Derecha",
        (_, "output_done") => "Prueba de salida completada.",
        (_, "mic_test") => "Probar micrófono",
        (_, "mic_pick") => "Micrófono para la prueba",
        (_, "mic_level") => "Nivel del micrófono en vivo",
        (_, "mic_level_detail") => {
            "Habla con normalidad mientras la medición esté activa. Pulsa Detener medición cuando termines."
        }
        (_, "measure") => "Iniciar medición",
        (_, "measure_stop") => "Detener medición",
        (_, "record") => "Grabar prueba",
        (_, "record_again") => "Grabar de nuevo",
        (_, "recording_button") => "Grabando…",
        (_, "playing_button") => "Reproduciendo…",
        (_, "play") => "Reproducir prueba",
        (_, "voice_detail") => {
            "Graba una muestra privada breve. Al reproducirla, la grabación temporal se elimina automáticamente."
        }
        (_, "recorded") => "Prueba temporal del micrófono grabada.",
        (_, "played") => "Prueba del micrófono reproducida y eliminada.",
        _ => "Korunix",
    }
}

enum EventoMedidorMicrofono {
    Nivel(u8),
    Terminado(Result<(), String>),
}

fn medir_microfono_gui(
    estado: &Estado,
    source_id: u32,
    barra: &gtk::LevelBar,
    porcentaje: &gtk::Label,
    continuar: Rc<Cell<bool>>,
) -> Result<(), String> {
    if estado.ocupado.replace(true) {
        return Err("Korunix ya está realizando otra operación.".to_string());
    }

    barra.set_value(0.0);
    porcentaje.set_text("Midiendo…");

    let contexto = glib::MainContext::default();
    let mut resultado_final = Ok(());

    while continuar.get() {
        let motor = estado.motor.clone();
        let raiz = estado.raiz.clone();
        let id = source_id.to_string();
        let (emisor, receptor) = mpsc::channel::<EventoMedidorMicrofono>();

        thread::spawn(move || {
            let resultado = (|| -> Result<(), String> {
                let mut hijo = Command::new(motor)
                    .args([
                        "media",
                        "mic",
                        "meter",
                        &id,
                        "--seconds",
                        "1",
                        "--yes",
                        "--json",
                    ])
                    .current_dir(&raiz)
                    .env("KORUNIX_ROOT", &raiz)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|error| {
                        format!("No pude iniciar la medición del micrófono: {error}")
                    })?;

                let stderr = hijo
                    .stderr
                    .take()
                    .ok_or_else(|| "No pude leer el nivel del micrófono.".to_string())?;

                let mut detalles = Vec::<String>::new();
                let mut hubo_nivel = false;

                for linea in BufReader::new(stderr).lines() {
                    let linea =
                        linea.map_err(|error| format!("No pude leer el micrófono: {error}"))?;
                    if let Some(resto) = linea.strip_prefix("KORUNIX_MIC_LEVEL\t") {
                        if let Ok(nivel) = resto.trim().parse::<u8>() {
                            hubo_nivel = true;
                            let _ = emisor.send(EventoMedidorMicrofono::Nivel(nivel.min(100)));
                            continue;
                        }
                    }
                    if !linea.trim().is_empty() {
                        detalles.push(linea);
                    }
                }

                let estado_hijo = hijo
                    .wait()
                    .map_err(|error| format!("No pude esperar la medición: {error}"))?;

                if !estado_hijo.success() && !hubo_nivel {
                    return Err(if detalles.is_empty() {
                        "No se pudo medir el micrófono.".to_string()
                    } else {
                        detalles.join("\n")
                    });
                }

                Ok(())
            })();

            let _ = emisor.send(EventoMedidorMicrofono::Terminado(resultado));
        });

        loop {
            match receptor.try_recv() {
                Ok(EventoMedidorMicrofono::Nivel(nivel)) => {
                    barra.set_value(f64::from(nivel) / 100.0);
                    porcentaje.set_text(&format!("{nivel}%"));
                }
                Ok(EventoMedidorMicrofono::Terminado(resultado)) => {
                    if let Err(error) = resultado {
                        if continuar.get() {
                            resultado_final = Err(error);
                        }
                    }
                    break;
                }
                Err(TryRecvError::Empty) => {
                    while contexto.pending() {
                        contexto.iteration(false);
                    }
                    thread::sleep(Duration::from_millis(12));
                }
                Err(TryRecvError::Disconnected) => {
                    if continuar.get() {
                        resultado_final =
                            Err("La medición del micrófono terminó inesperadamente.".to_string());
                    }
                    break;
                }
            }
        }

        if resultado_final.is_err() {
            break;
        }
    }

    continuar.set(false);
    estado.ocupado.set(false);
    resultado_final
}

fn ejecutar_prueba_salida_gui(estado: &Estado, selector: &adw::ComboRow, ids: &[u32], canal: &str) {
    let Some(id) = ids.get(selector.selected() as usize).copied() else {
        return;
    };

    let id = id.to_string();
    match ejecutar_json(
        estado,
        &[
            "media",
            "audio",
            "test-output",
            &id,
            "--channel",
            canal,
            "--seconds",
            "2",
            "--yes",
            "--json",
        ],
    ) {
        Ok(_) => mostrar_exito(
            estado,
            texto_prueba_multimedia(estado.idioma, "output_done"),
        ),
        Err(error) => mostrar_error(estado, error),
    }
}

enum EventoCamara {
    Frame(Vec<u8>),
    Terminado(Result<(), String>),
}

fn abrir_prueba_camara_gui(estado: Rc<Estado>, device: String, titulo: String) {
    const WIDTH: i32 = 640;
    const HEIGHT: i32 = 360;
    const STRIDE: usize = WIDTH as usize * 4;
    const FRAME_SIZE: usize = STRIDE * HEIGHT as usize;

    if estado.camara_preview_activa.replace(true) {
        estado.toast.add_toast(adw::Toast::new(texto_prueba_camara(
            estado.idioma,
            "one_preview",
        )));
        return;
    }

    let ventana = adw::Window::new();
    ventana.set_title(Some(&format!(
        "{} · {}",
        texto_prueba_camara(estado.idioma, "title"),
        titulo
    )));
    ventana.set_default_size(720, 500);
    ventana.set_modal(false);

    if let Some(parent) = estado
        .stack
        .root()
        .and_then(|root| root.downcast::<gtk::Window>().ok())
    {
        ventana.set_transient_for(Some(&parent));
    }

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar.add_top_bar(&header);

    let contenido = gtk::Box::new(gtk::Orientation::Vertical, 12);
    contenido.set_margin_start(18);
    contenido.set_margin_end(18);
    contenido.set_margin_top(18);
    contenido.set_margin_bottom(18);

    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_size_request(WIDTH, HEIGHT);
    picture.add_css_class("card");

    let estado_preview = gtk::Label::new(Some(texto_prueba_camara(estado.idioma, "starting")));
    estado_preview.set_xalign(0.0);
    estado_preview.add_css_class("dim-label");

    contenido.append(&picture);
    contenido.append(&estado_preview);
    toolbar.set_content(Some(&contenido));
    ventana.set_content(Some(&toolbar));

    let mut hijo = match Command::new(&estado.motor)
        .args([
            "media", "camera", "stream", &device, "--width", "640", "--height", "360",
        ])
        .current_dir(&estado.raiz)
        .env("KORUNIX_ROOT", &estado.raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(hijo) => hijo,
        Err(error) => {
            mostrar_error(&estado, format!("No pude iniciar la cámara: {error}"));
            return;
        }
    };

    let Some(mut stdout) = hijo.stdout.take() else {
        let _ = hijo.kill();
        let _ = hijo.wait();
        estado.camara_preview_activa.set(false);
        mostrar_error(&estado, "No pude recibir la imagen de la cámara.");
        return;
    };
    let stderr = hijo.stderr.take();

    let proceso = Arc::new(Mutex::new(hijo));
    let cerrado = Arc::new(AtomicBool::new(false));
    let (emisor, receptor) = mpsc::sync_channel::<EventoCamara>(2);

    {
        let proceso = Arc::clone(&proceso);
        let cerrado = Arc::clone(&cerrado);
        thread::spawn(move || {
            let mut detalle = String::new();
            let mut frame = vec![0u8; FRAME_SIZE];

            loop {
                match stdout.read_exact(&mut frame) {
                    Ok(()) => {
                        if emisor.try_send(EventoCamara::Frame(frame.clone())).is_err() {
                            // Si GTK aún no consumió el cuadro anterior, se descarta este.
                        }
                    }
                    Err(error) => {
                        if !cerrado.load(Ordering::Relaxed) {
                            detalle = format!("La cámara dejó de entregar imagen: {error}");
                        }
                        break;
                    }
                }
            }

            if let Some(mut stderr) = stderr {
                let mut texto = String::new();
                let _ = stderr.read_to_string(&mut texto);
                if !texto.trim().is_empty() {
                    detalle = texto.trim().to_string();
                }
            }

            let status = proceso.lock().ok().and_then(|mut hijo| hijo.wait().ok());

            let resultado = if cerrado.load(Ordering::Relaxed) {
                Ok(())
            } else if status.map(|status| status.success()) == Some(true) {
                Ok(())
            } else if detalle.is_empty() {
                Err("La previsualización de la cámara terminó inesperadamente.".to_string())
            } else {
                Err(detalle)
            };

            let _ = emisor.send(EventoCamara::Terminado(resultado));
        });
    }

    {
        let proceso = Arc::clone(&proceso);
        let cerrado = Arc::clone(&cerrado);
        let estado_cierre = Rc::clone(&estado);
        ventana.connect_close_request(move |_| {
            cerrado.store(true, Ordering::Relaxed);
            if let Ok(mut hijo) = proceso.lock() {
                let _ = hijo.kill();
                let _ = hijo.wait();
            }
            estado_cierre.camara_preview_activa.set(false);
            glib::Propagation::Proceed
        });
    }

    {
        let proceso = Arc::clone(&proceso);
        let cerrado = Arc::clone(&cerrado);
        let estado_destroy = Rc::clone(&estado);
        ventana.connect_destroy(move |_| {
            cerrado.store(true, Ordering::Relaxed);
            if let Ok(mut hijo) = proceso.lock() {
                let _ = hijo.kill();
                let _ = hijo.wait();
            }
            estado_destroy.camara_preview_activa.set(false);
        });
    }

    {
        let estado_evento = Rc::clone(&estado);
        let picture = picture.clone();
        let estado_preview = estado_preview.clone();
        glib::timeout_add_local(Duration::from_millis(33), move || {
            let mut ultimo = None::<Vec<u8>>;
            loop {
                match receptor.try_recv() {
                    Ok(EventoCamara::Frame(frame)) => ultimo = Some(frame),
                    Ok(EventoCamara::Terminado(resultado)) => {
                        estado_evento.camara_preview_activa.set(false);
                        if let Err(error) = resultado {
                            let ocupada = error.to_ascii_lowercase().contains("busy")
                                || error.to_ascii_lowercase().contains("resource busy");
                            let mensaje = if ocupada {
                                texto_prueba_camara(estado_evento.idioma, "busy").to_string()
                            } else {
                                error
                            };
                            estado_preview.set_text(&mensaje);
                            if !ocupada {
                                estado_evento.toast.add_toast(adw::Toast::new(&mensaje));
                            }
                        }
                        return glib::ControlFlow::Break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return glib::ControlFlow::Break,
                }
            }

            if let Some(frame) = ultimo {
                let bytes = glib::Bytes::from_owned(frame);
                let textura = gtk::gdk::MemoryTexture::new(
                    WIDTH,
                    HEIGHT,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &bytes,
                    STRIDE,
                );
                picture.set_paintable(Some(&textura));
                estado_preview.set_text(texto_prueba_camara(estado_evento.idioma, "live"));
            }

            glib::ControlFlow::Continue
        });
    }

    ventana.present();
}

fn texto_prueba_camara(idioma: Idioma, clave: &str) -> &'static str {
    match (idioma, clave) {
        (Idioma::Ingles, "title") => "Camera test",
        (Idioma::BelarusLatino, "title") => "Test kamjery",
        (Idioma::Belarus, "title") => "Тэст камеры",
        (Idioma::Catalan, "title") => "Prova de càmera",
        (Idioma::Checo, "title") => "Test fotoaparátu",
        (Idioma::Aleman, "title") => "Kameratest",
        (Idioma::Frances, "title") => "Test de caméra",
        (Idioma::Gallego, "title") => "Proba da cámara",
        (Idioma::Italiano, "title") => "Prova della fotocamera",
        (Idioma::Coreano, "title") => "카메라 테스트",
        (Idioma::Kurdo, "title") => "Testa kamerayê",
        (Idioma::Neerlandes, "title") => "Cameratest",
        (Idioma::NoruegoNynorsk, "title") => "Kamera test",
        (Idioma::Polaco, "title") => "Test aparatu",
        (Idioma::PortuguesBrasil, "title") => "Teste de câmera",
        (Idioma::Ruso, "title") => "Тест камеры",
        (Idioma::Sueco, "title") => "Kameratest",
        (Idioma::Turco, "title") => "Kamera testi",
        (Idioma::Ucraniano, "title") => "Тест камери",
        (Idioma::Vietnamita, "title") => "Kiểm tra máy ảnh",
        (Idioma::ChinoSimplificado, "title") => "相机测试",
        (Idioma::Ingles, "detail") => "Opens a temporary preview. Korunix does not save video.",
        (Idioma::BelarusLatino, "detail") => "Adkryvaje časovy papjaredni prahłjad. Korunix nje zachoŭvaje videa.",
        (Idioma::Belarus, "detail") => "Адкрывае часовы папярэдні прагляд. Korunix не захоўвае відэа.",
        (Idioma::Catalan, "detail") => "Obre una vista prèvia temporal. Korunix no desa el vídeo.",
        (Idioma::Checo, "detail") => "Otevře dočasný náhled. Korunix neukládá video.",
        (Idioma::Aleman, "detail") => "Öffnet eine temporäre Vorschau. Korunix speichert keine Videos.",
        (Idioma::Frances, "detail") => "Ouvre un aperçu temporaire. Korunix n'enregistre pas la vidéo.",
        (Idioma::Gallego, "detail") => "Abre unha vista previa temporal. Korunix non garda o vídeo.",
        (Idioma::Italiano, "detail") => "Apre un'anteprima temporanea. Korunix non salva i video.",
        (Idioma::Coreano, "detail") => "임시 미리보기를 엽니다. Korunix는 영상을 저장하지 않습니다.",
        (Idioma::Kurdo, "detail") => "Pêşdîtinek demkî vedike. Korunix vîdyoyê tomar nake.",
        (Idioma::Neerlandes, "detail") => "Opent een tijdelijk voorbeeld. Korunix slaat geen video op.",
        (Idioma::NoruegoNynorsk, "detail") => "Åpner en midlertidig forhåndsvisning. Korunix lagrer ikke video.",
        (Idioma::Polaco, "detail") => "Otwiera tymczasowy podgląd. Korunix nie zapisuje wideo.",
        (Idioma::PortuguesBrasil, "detail") => "Abre uma visualização temporária. Korunix não salva vídeo.",
        (Idioma::Ruso, "detail") => "Открывает временный предварительный просмотр. Korunix не сохраняет видео.",
        (Idioma::Sueco, "detail") => "Öppnar en tillfällig förhandsvisning. Korunix sparar inte video.",
        (Idioma::Turco, "detail") => "Geçici bir önizleme açar. Korunix videoyu kaydetmiyor.",
        (Idioma::Ucraniano, "detail") => "Відкриває тимчасовий попередній перегляд. Korunix не зберігає відео.",
        (Idioma::Vietnamita, "detail") => "Mở bản xem trước tạm thời. Korunix không lưu video.",
        (Idioma::ChinoSimplificado, "detail") => "打开临时预览。 Korunix 不保存视频。",
        (Idioma::Ingles, "action") => "Test camera",
        (Idioma::BelarusLatino, "action") => "Test kamjery",
        (Idioma::Belarus, "action") => "Тэст камеры",
        (Idioma::Catalan, "action") => "Càmera de prova",
        (Idioma::Checo, "action") => "Testovací kamera",
        (Idioma::Aleman, "action") => "Kamera testen",
        (Idioma::Frances, "action") => "Caméra de test",
        (Idioma::Gallego, "action") => "Cámara de proba",
        (Idioma::Italiano, "action") => "Prova la fotocamera",
        (Idioma::Coreano, "action") => "테스트 카메라",
        (Idioma::Kurdo, "action") => "Kamera testê",
        (Idioma::Neerlandes, "action") => "Testcamera",
        (Idioma::NoruegoNynorsk, "action") => "Test kamera",
        (Idioma::Polaco, "action") => "Aparat testowy",
        (Idioma::PortuguesBrasil, "action") => "Câmera de teste",
        (Idioma::Ruso, "action") => "Тестовая камера",
        (Idioma::Sueco, "action") => "Testa kamera",
        (Idioma::Turco, "action") => "Kamerayı test edin",
        (Idioma::Ucraniano, "action") => "Тестова камера",
        (Idioma::Vietnamita, "action") => "Máy ảnh thử nghiệm",
        (Idioma::ChinoSimplificado, "action") => "测试相机",
        (Idioma::Ingles, "done") => "Camera test finished.",
        (Idioma::BelarusLatino, "done") => "Test kamjery zavjeršany.",
        (Idioma::Belarus, "done") => "Тэст камеры завершаны.",
        (Idioma::Catalan, "done") => "S'ha acabat la prova de la càmera.",
        (Idioma::Checo, "done") => "Test fotoaparátu dokončen.",
        (Idioma::Aleman, "done") => "Kameratest abgeschlossen.",
        (Idioma::Frances, "done") => "Test de la caméra terminé.",
        (Idioma::Gallego, "done") => "Rematou a proba da cámara.",
        (Idioma::Italiano, "done") => "Test della fotocamera terminato.",
        (Idioma::Coreano, "done") => "카메라 테스트가 완료되었습니다.",
        (Idioma::Kurdo, "done") => "Testa kamerayê qediya.",
        (Idioma::Neerlandes, "done") => "Cameratest voltooid.",
        (Idioma::NoruegoNynorsk, "done") => "Kameratesten er ferdig.",
        (Idioma::Polaco, "done") => "Test aparatu zakończony.",
        (Idioma::PortuguesBrasil, "done") => "Teste de câmera concluído.",
        (Idioma::Ruso, "done") => "Тест камеры завершен.",
        (Idioma::Sueco, "done") => "Kameratestet avslutat.",
        (Idioma::Turco, "done") => "Kamera testi tamamlandı.",
        (Idioma::Ucraniano, "done") => "Тест камери завершено.",
        (Idioma::Vietnamita, "done") => "Kiểm tra máy ảnh đã hoàn tất.",
        (Idioma::ChinoSimplificado, "done") => "相机测试完成。",
        (Idioma::Ingles, "starting") => "Starting camera…",
        (Idioma::BelarusLatino, "starting") => "Zapusk kamjery…",
        (Idioma::Belarus, "starting") => "Запуск камеры…",
        (Idioma::Catalan, "starting") => "S'està iniciant la càmera...",
        (Idioma::Checo, "starting") => "Spouštění fotoaparátu…",
        (Idioma::Aleman, "starting") => "Kamera wird gestartet…",
        (Idioma::Frances, "starting") => "Démarrage de la caméra…",
        (Idioma::Gallego, "starting") => "Iniciando a cámara...",
        (Idioma::Italiano, "starting") => "Avvio della fotocamera…",
        (Idioma::Coreano, "starting") => "카메라 시작 중…",
        (Idioma::Kurdo, "starting") => "Kamera dest pê dike…",
        (Idioma::Neerlandes, "starting") => "Camera starten…",
        (Idioma::NoruegoNynorsk, "starting") => "Starter kamera …",
        (Idioma::Polaco, "starting") => "Uruchamiam kamerę…",
        (Idioma::PortuguesBrasil, "starting") => "Iniciando a câmera…",
        (Idioma::Ruso, "starting") => "Запуск камеры…",
        (Idioma::Sueco, "starting") => "Startar kamera...",
        (Idioma::Turco, "starting") => "Kamera başlatılıyor…",
        (Idioma::Ucraniano, "starting") => "Запуск камери…",
        (Idioma::Vietnamita, "starting") => "Đang khởi động máy ảnh…",
        (Idioma::ChinoSimplificado, "starting") => "启动相机...",
        (Idioma::Ingles, "live") => "Live preview · nothing is being recorded",
        (Idioma::BelarusLatino, "live") => "Anłajn-prahłjad · ničoha nje zapisvajecca",
        (Idioma::Belarus, "live") => "Анлайн-прагляд · нічога не запісваецца",
        (Idioma::Catalan, "live") => "Vista prèvia en directe · no s'està gravant res",
        (Idioma::Checo, "live") => "Živý náhled · nic se nenahrává",
        (Idioma::Aleman, "live") => "Live-Vorschau · Es wird nichts aufgezeichnet",
        (Idioma::Frances, "live") => "Aperçu en direct · rien n'est enregistré",
        (Idioma::Gallego, "live") => "Vista previa en directo · non se está gravando nada",
        (Idioma::Italiano, "live") => "Anteprima dal vivo · non viene registrato nulla",
        (Idioma::Coreano, "live") => "실시간 미리보기 · 녹화 중인 항목이 없습니다.",
        (Idioma::Kurdo, "live") => "Pêşdîtina zindî · tiştek nayê tomar kirin",
        (Idioma::Neerlandes, "live") => "Live preview · er wordt niets opgenomen",
        (Idioma::NoruegoNynorsk, "live") => "Live forhåndsvisning · ingenting blir tatt opp",
        (Idioma::Polaco, "live") => "Podgląd na żywo · nic nie jest nagrywane",
        (Idioma::PortuguesBrasil, "live") => "Visualização ao vivo · nada está sendo gravado",
        (Idioma::Ruso, "live") => "Предварительный просмотр · ничего не записывается",
        (Idioma::Sueco, "live") => "Liveförhandsvisning · ingenting spelas in",
        (Idioma::Turco, "live") => "Canlı önizleme · hiçbir şey kaydedilmiyor",
        (Idioma::Ucraniano, "live") => "Попередній перегляд у реальному часі · нічого не записується",
        (Idioma::Vietnamita, "live") => "Xem trước trực tiếp · không có gì được ghi lại",
        (Idioma::ChinoSimplificado, "live") => "实时预览 · 没有录制任何内容",
        (Idioma::Ingles, "busy") => "This camera is already in use. Stop the other preview or close the app using it.",
        (Idioma::BelarusLatino, "busy") => "Hetaja kamjera ŭžo vykarystoŭvajecca. Spynicje inšy papjaredni prahłjad abo začynicje prahramu, jakaja vykarystoŭvaje jaho.",
        (Idioma::Belarus, "busy") => "Гэтая камера ўжо выкарыстоўваецца. Спыніце іншы папярэдні прагляд або зачыніце праграму, якая выкарыстоўвае яго.",
        (Idioma::Catalan, "busy") => "Aquesta càmera ja està en ús. Atureu l'altra vista prèvia o tanqueu l'aplicació utilitzant-la.",
        (Idioma::Checo, "busy") => "Tato kamera se již používá. Zastavte druhý náhled nebo zavřete aplikaci, která jej používá.",
        (Idioma::Aleman, "busy") => "Diese Kamera ist bereits im Einsatz. Stoppen Sie die andere Vorschau oder schließen Sie die App, die sie verwendet.",
        (Idioma::Frances, "busy") => "Cette caméra est déjà utilisée. Arrêtez l'autre aperçu ou fermez l'application qui l'utilise.",
        (Idioma::Gallego, "busy") => "Esta cámara xa está en uso. Detén a outra vista previa ou pecha a aplicación usándoa.",
        (Idioma::Italiano, "busy") => "Questa fotocamera è già in uso. Interrompi l'altra anteprima o chiudi l'app che la utilizza.",
        (Idioma::Coreano, "busy") => "이 카메라는 이미 사용 중입니다. 다른 미리보기를 중지하거나 이를 사용하여 앱을 닫습니다.",
        (Idioma::Kurdo, "busy") => "Ev kamera jixwe tê bikaranîn. Pêşdîtina din rawestînin an sepanê bi karanîna wê bigire.",
        (Idioma::Neerlandes, "busy") => "Deze camera is al in gebruik. Stop het andere voorbeeld of sluit de app ermee.",
        (Idioma::NoruegoNynorsk, "busy") => "Dette kameraet er allerede i bruk. Stopp den andre forhåndsvisningen eller lukk appen ved å bruke den.",
        (Idioma::Polaco, "busy") => "Ten aparat jest już używany. Zatrzymaj inny podgląd lub zamknij aplikację, która go używa.",
        (Idioma::PortuguesBrasil, "busy") => "Esta câmera já está em uso. Pare a outra visualização ou feche o aplicativo que a utiliza.",
        (Idioma::Ruso, "busy") => "Эта камера уже используется. Остановите другой предварительный просмотр или закройте приложение, использующее его.",
        (Idioma::Sueco, "busy") => "Den här kameran används redan. Stoppa den andra förhandsgranskningen eller stäng appen med den.",
        (Idioma::Turco, "busy") => "Bu kamera zaten kullanılıyor. Diğer önizlemeyi durdurun veya onu kullanarak uygulamayı kapatın.",
        (Idioma::Ucraniano, "busy") => "Ця камера вже використовується. Зупиніть інший попередній перегляд або закрийте програму, яка його використовує.",
        (Idioma::Vietnamita, "busy") => "Máy ảnh này đã được sử dụng. Dừng bản xem trước khác hoặc đóng ứng dụng bằng cách sử dụng nó.",
        (Idioma::ChinoSimplificado, "busy") => "该相机已投入使用。停止其他预览或关闭使用它的应用程序。",
        (Idioma::Ingles, "one_preview") => "A camera test is already open.",
        (Idioma::BelarusLatino, "one_preview") => "Test kamjery ŭžo adkryty.",
        (Idioma::Belarus, "one_preview") => "Тэст камеры ўжо адкрыты.",
        (Idioma::Catalan, "one_preview") => "Ja està oberta una prova de càmera.",
        (Idioma::Checo, "one_preview") => "Kamerový test je již otevřen.",
        (Idioma::Aleman, "one_preview") => "Ein Kameratest ist bereits eröffnet.",
        (Idioma::Frances, "one_preview") => "Un test caméra est déjà ouvert.",
        (Idioma::Gallego, "one_preview") => "Xa está aberta unha proba da cámara.",
        (Idioma::Italiano, "one_preview") => "Un test della fotocamera è già aperto.",
        (Idioma::Coreano, "one_preview") => "카메라 테스트가 이미 열려 있습니다.",
        (Idioma::Kurdo, "one_preview") => "Testek kamerayê jixwe vekirî ye.",
        (Idioma::Neerlandes, "one_preview") => "Er is al een cameratest geopend.",
        (Idioma::NoruegoNynorsk, "one_preview") => "En kameratest er allerede åpen.",
        (Idioma::Polaco, "one_preview") => "Test aparatu jest już otwarty.",
        (Idioma::PortuguesBrasil, "one_preview") => "Um teste de câmera já está aberto.",
        (Idioma::Ruso, "one_preview") => "Тест камеры уже открыт.",
        (Idioma::Sueco, "one_preview") => "Ett kameratest är redan öppet.",
        (Idioma::Turco, "one_preview") => "Bir kamera testi zaten açık.",
        (Idioma::Ucraniano, "one_preview") => "Тест камери вже відкритий.",
        (Idioma::Vietnamita, "one_preview") => "Một thử nghiệm máy ảnh đã được mở.",
        (Idioma::ChinoSimplificado, "one_preview") => "相机测试已经开放。",
        (Idioma::Hungaro, "title") => "Kamera tesztje",
        (Idioma::Hungaro, "detail") => "Ideiglenes előnézetet nyit. A Korunix nem ment videót.",
        (Idioma::Hungaro, "action") => "Kamera tesztelése",
        (Idioma::Hungaro, "done") => "A kamera tesztje befejeződött.",
        (Idioma::Hungaro, "starting") => "Kamera indítása…",
        (Idioma::Hungaro, "live") => "Élő előnézet · nincs videófelvétel",
        (Idioma::Hungaro, "busy") => "A kamerát már használja egy másik alkalmazás. Állítsd le a másik előnézetet.",
        (Idioma::Hungaro, "one_preview") => "Már nyitva van egy kamerateszt.",
        (_, "title") => "Prueba de cámara",
        (_, "detail") => "Abre una vista temporal. Korunix no guarda vídeo.",
        (_, "action") => "Probar cámara",
        (_, "done") => "Prueba de cámara finalizada.",
        (_, "starting") => "Iniciando cámara…",
        (_, "live") => "Vista previa en vivo · no se está grabando vídeo",
        (_, "busy") => "Esta cámara ya está en uso. Detén la otra vista o cierra la aplicación que la está usando.",
        (_, "one_preview") => "Ya hay una prueba de cámara abierta.",
        _ => "Korunix",
    }
}

fn modos_camara_humanos(formatos: &Value) -> Vec<String> {
    let mut modos = Vec::<(u64, u64, f64)>::new();

    for formato in formatos.as_array().into_iter().flatten() {
        let tamanos = formato
            .get("sizes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for tamano in tamanos {
            let Some(ancho) = tamano.get("width").and_then(Value::as_u64) else {
                continue;
            };
            let Some(alto) = tamano.get("height").and_then(Value::as_u64) else {
                continue;
            };

            let max_fps = tamano
                .get("fps")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_f64)
                .fold(0.0_f64, f64::max);

            if let Some(actual) = modos.iter_mut().find(|(w, h, _)| *w == ancho && *h == alto) {
                actual.2 = actual.2.max(max_fps);
            } else {
                modos.push((ancho, alto, max_fps));
            }
        }
    }

    modos.sort_by(|a, b| {
        (b.0 * b.1)
            .cmp(&(a.0 * a.1))
            .then_with(|| b.2.total_cmp(&a.2))
    });

    modos
        .into_iter()
        .take(6)
        .map(|(ancho, alto, fps)| {
            if fps > 0.0 {
                let fps = if (fps.fract()).abs() < 0.001 {
                    format!("{fps:.0}")
                } else {
                    format!("{fps:.1}")
                };
                format!("{ancho}×{alto} · {fps} FPS")
            } else {
                format!("{ancho}×{alto}")
            }
        })
        .collect()
}

fn resumen_modos_camara(camara: &Value) -> String {
    let modos = camara
        .get("formats")
        .map(modos_camara_humanos)
        .unwrap_or_default();

    if modos.is_empty() {
        String::new()
    } else {
        modos.join(", ")
    }
}

fn pagina_multimedia(estado: Rc<Estado>, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let audio = datos.get("audio").cloned().unwrap_or(Value::Null);

    let sinks = audio
        .get("sinks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let sources = audio
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let grupo_salida = adw::PreferencesGroup::new();
    grupo_salida.set_title(texto(estado.idioma, "output"));

    if sinks.is_empty() {
        grupo_salida.add(&fila(
            texto(estado.idioma, "status"),
            texto(estado.idioma, "empty"),
        ));
    } else {
        let predeterminado = audio
            .pointer("/defaults/sinkId")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok());

        let ids = sinks
            .iter()
            .filter_map(|node| {
                node.get("id")
                    .and_then(Value::as_u64)
                    .and_then(|id| u32::try_from(id).ok())
            })
            .collect::<Vec<_>>();

        let etiquetas = sinks
            .iter()
            .filter_map(|node| {
                let id = node.get("id").and_then(Value::as_u64)?;
                let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
                Some(descripcion_nodo_audio(
                    estado.idioma,
                    &audio,
                    "sink",
                    id,
                    nombre,
                ))
            })
            .collect::<Vec<_>>();

        if ids.len() == etiquetas.len() && !ids.is_empty() {
            let selector = adw::ComboRow::new();
            selector.set_title(texto(estado.idioma, "output"));
            selector.set_model(Some(&modelo_cadenas(&etiquetas)));

            let indice = predeterminado
                .and_then(|id| ids.iter().position(|candidato| *candidato == id))
                .unwrap_or(0) as u32;

            selector.set_selected(indice);
            grupo_salida.add(&selector);

            let ids = Rc::new(ids);
            let estado_default = Rc::clone(&estado);

            selector.connect_selected_notify(move |selector| {
                let Some(id) = ids.get(selector.selected() as usize).copied() else {
                    return;
                };

                let id_texto = id.to_string();
                if let Err(error) = ejecutar_motor(
                    &estado_default,
                    &["media", "audio", "default", "sink", &id_texto],
                ) {
                    mostrar_error(&estado_default, error);
                }
            });
        }

        for node in &sinks {
            let Some(id) = node.get("id").and_then(Value::as_u64) else {
                continue;
            };

            let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
            let titulo = descripcion_nodo_audio(estado.idioma, &audio, "sink", id, nombre);
            let row = adw::ActionRow::new();
            row.set_title(&titulo);

            let es_predeterminado = u32::try_from(id)
                .ok()
                .map(|id| Some(id) == predeterminado)
                .unwrap_or(false);

            let subtitulo = if es_predeterminado {
                match estado.idioma {
                    Idioma::Ingles => "Default · Volume",
                    Idioma::BelarusLatino => "Pa zmaŭčanni · Ab'jom",
                    Idioma::Belarus => "Па змаўчанні · Аб'ём",
                    Idioma::Catalan => "Per defecte · Volum",
                    Idioma::Checo => "Výchozí · Hlasitost",
                    Idioma::Aleman => "Standard · Lautstärke",
                    Idioma::Frances => "Par défaut · Volume",
                    Idioma::Gallego => "Predeterminado · Volume",
                    Idioma::Italiano => "Predefinito · Volume",
                    Idioma::Coreano => "기본값 · 볼륨",
                    Idioma::Kurdo => "Bingehîn · Volume",
                    Idioma::Neerlandes => "Standaard · Volume",
                    Idioma::NoruegoNynorsk => "Standard · Volum",
                    Idioma::Polaco => "Domyślne · Głośność",
                    Idioma::PortuguesBrasil => "Padrão · Volume",
                    Idioma::Ruso => "По умолчанию · Объем",
                    Idioma::Sueco => "Standard · Volym",
                    Idioma::Turco => "Varsayılan · Hacim",
                    Idioma::Ucraniano => "За замовчуванням · Гучність",
                    Idioma::Vietnamita => "Mặc định · Âm lượng",
                    Idioma::ChinoSimplificado => "默认·音量",
                    Idioma::Hungaro => "Alapértelmezett · Hangerő",
                    Idioma::Espanol => "Predeterminado · Volumen",
                }
            } else {
                texto(estado.idioma, "volume")
            };

            row.set_subtitle(subtitulo);

            let escala = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 150.0, 1.0);
            escala.set_width_request(120);
            escala.set_draw_value(false);
            escala.set_valign(gtk::Align::Center);
            escala.set_value(node.get("volume").and_then(Value::as_f64).unwrap_or(0.0) * 100.0);
            row.add_suffix(&escala);

            let silencio = gtk::ToggleButton::with_label(texto(estado.idioma, "mute"));
            silencio.set_active(node.get("muted").and_then(Value::as_bool).unwrap_or(false));
            silencio.set_valign(gtk::Align::Center);
            row.add_suffix(&silencio);
            grupo_salida.add(&row);

            let id_texto = id.to_string();
            let estado_volumen = Rc::clone(&estado);
            escala.connect_value_changed(move |escala| {
                let volumen = format!("{:.0}%", escala.value());
                if let Err(error) = ejecutar_motor(
                    &estado_volumen,
                    &["media", "audio", "volume", &id_texto, &volumen],
                ) {
                    mostrar_error(&estado_volumen, error);
                }
            });

            let id_texto = id.to_string();
            let estado_silencio = Rc::clone(&estado);
            silencio.connect_toggled(move |boton| {
                let valor = if boton.is_active() { "1" } else { "0" };
                if let Err(error) = ejecutar_motor(
                    &estado_silencio,
                    &["media", "audio", "mute", &id_texto, valor],
                ) {
                    mostrar_error(&estado_silencio, error);
                }
            });
        }
    }

    pagina.add(&grupo_salida);

    let grupo_prueba_salida = adw::PreferencesGroup::new();
    grupo_prueba_salida.set_title(texto_prueba_multimedia(estado.idioma, "output_test"));

    if !sinks.is_empty() {
        let ids = Rc::new(
            sinks
                .iter()
                .filter_map(|node| {
                    node.get("id")
                        .and_then(Value::as_u64)
                        .and_then(|id| u32::try_from(id).ok())
                })
                .collect::<Vec<_>>(),
        );

        let etiquetas = sinks
            .iter()
            .filter_map(|node| {
                let id = node.get("id").and_then(Value::as_u64)?;
                let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
                Some(descripcion_nodo_audio(
                    estado.idioma,
                    &audio,
                    "sink",
                    id,
                    nombre,
                ))
            })
            .collect::<Vec<_>>();

        let referencias = etiquetas.iter().map(String::as_str).collect::<Vec<_>>();
        let modelo = gtk::StringList::new(&referencias);
        let selector = adw::ComboRow::new();
        selector.set_title(texto_prueba_multimedia(estado.idioma, "output_pick"));
        selector.set_model(Some(&modelo));

        let predeterminado = audio
            .pointer("/defaults/sinkId")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok());

        if let Some(indice) = ids.iter().position(|id| Some(*id) == predeterminado) {
            selector.set_selected(indice as u32);
        }

        grupo_prueba_salida.add(&selector);

        let canales = adw::ActionRow::new();
        canales.set_title(texto_prueba_multimedia(estado.idioma, "output_test"));
        canales.set_subtitle(texto_prueba_multimedia(estado.idioma, "output_detail"));

        let botones = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        botones.set_valign(gtk::Align::Center);
        botones.set_halign(gtk::Align::End);
        botones.set_hexpand(false);
        let ambos = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "both"));
        ambos.add_css_class("suggested-action");
        ambos.set_valign(gtk::Align::Center);
        let izquierda = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "left"));
        izquierda.set_valign(gtk::Align::Center);
        let derecha = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "right"));
        derecha.set_valign(gtk::Align::Center);

        botones.append(&izquierda);
        botones.append(&ambos);
        botones.append(&derecha);
        canales.add_suffix(&botones);
        grupo_prueba_salida.add(&canales);

        {
            let estado_prueba = Rc::clone(&estado);
            let selector_prueba = selector.clone();
            let ids_prueba = Rc::clone(&ids);
            ambos.connect_clicked(move |_| {
                ejecutar_prueba_salida_gui(
                    &estado_prueba,
                    &selector_prueba,
                    ids_prueba.as_slice(),
                    "both",
                );
            });
        }

        {
            let estado_prueba = Rc::clone(&estado);
            let selector_prueba = selector.clone();
            let ids_prueba = Rc::clone(&ids);
            izquierda.connect_clicked(move |_| {
                ejecutar_prueba_salida_gui(
                    &estado_prueba,
                    &selector_prueba,
                    ids_prueba.as_slice(),
                    "left",
                );
            });
        }

        {
            let estado_prueba = Rc::clone(&estado);
            let selector_prueba = selector.clone();
            let ids_prueba = Rc::clone(&ids);
            derecha.connect_clicked(move |_| {
                ejecutar_prueba_salida_gui(
                    &estado_prueba,
                    &selector_prueba,
                    ids_prueba.as_slice(),
                    "right",
                );
            });
        }
    }

    pagina.add(&grupo_prueba_salida);

    let grupo_entrada = adw::PreferencesGroup::new();
    grupo_entrada.set_title(texto(estado.idioma, "input"));

    if sources.is_empty() {
        grupo_entrada.add(&fila(
            texto(estado.idioma, "status"),
            texto(estado.idioma, "empty"),
        ));
    } else {
        let predeterminado = audio
            .pointer("/defaults/sourceId")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok());

        let ids = sources
            .iter()
            .filter_map(|node| {
                node.get("id")
                    .and_then(Value::as_u64)
                    .and_then(|id| u32::try_from(id).ok())
            })
            .collect::<Vec<_>>();

        let etiquetas = sources
            .iter()
            .filter_map(|node| {
                let id = node.get("id").and_then(Value::as_u64)?;
                let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
                Some(descripcion_nodo_audio(
                    estado.idioma,
                    &audio,
                    "source",
                    id,
                    nombre,
                ))
            })
            .collect::<Vec<_>>();

        if ids.len() == etiquetas.len() && !ids.is_empty() {
            let selector = adw::ComboRow::new();
            selector.set_title(texto(estado.idioma, "input"));
            selector.set_model(Some(&modelo_cadenas(&etiquetas)));

            let indice = predeterminado
                .and_then(|id| ids.iter().position(|candidato| *candidato == id))
                .unwrap_or(0) as u32;

            selector.set_selected(indice);
            grupo_entrada.add(&selector);

            let ids = Rc::new(ids);
            let estado_default = Rc::clone(&estado);

            selector.connect_selected_notify(move |selector| {
                let Some(id) = ids.get(selector.selected() as usize).copied() else {
                    return;
                };

                let id_texto = id.to_string();
                if let Err(error) = ejecutar_motor(
                    &estado_default,
                    &["media", "audio", "default", "source", &id_texto],
                ) {
                    mostrar_error(&estado_default, error);
                }
            });
        }

        for node in &sources {
            let Some(id) = node.get("id").and_then(Value::as_u64) else {
                continue;
            };

            let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
            let titulo = descripcion_nodo_audio(estado.idioma, &audio, "source", id, nombre);

            let row = adw::ActionRow::new();
            row.set_title(&titulo);

            let es_predeterminado = u32::try_from(id)
                .ok()
                .map(|id| Some(id) == predeterminado)
                .unwrap_or(false);

            let subtitulo = if es_predeterminado {
                match estado.idioma {
                    Idioma::Ingles => "Default · Volume",
                    Idioma::BelarusLatino => "Pa zmaŭčanni · Ab'jom",
                    Idioma::Belarus => "Па змаўчанні · Аб'ём",
                    Idioma::Catalan => "Per defecte · Volum",
                    Idioma::Checo => "Výchozí · Hlasitost",
                    Idioma::Aleman => "Standard · Lautstärke",
                    Idioma::Frances => "Par défaut · Volume",
                    Idioma::Gallego => "Predeterminado · Volume",
                    Idioma::Italiano => "Predefinito · Volume",
                    Idioma::Coreano => "기본값 · 볼륨",
                    Idioma::Kurdo => "Bingehîn · Volume",
                    Idioma::Neerlandes => "Standaard · Volume",
                    Idioma::NoruegoNynorsk => "Standard · Volum",
                    Idioma::Polaco => "Domyślne · Głośność",
                    Idioma::PortuguesBrasil => "Padrão · Volume",
                    Idioma::Ruso => "По умолчанию · Объем",
                    Idioma::Sueco => "Standard · Volym",
                    Idioma::Turco => "Varsayılan · Hacim",
                    Idioma::Ucraniano => "За замовчуванням · Гучність",
                    Idioma::Vietnamita => "Mặc định · Âm lượng",
                    Idioma::ChinoSimplificado => "默认·音量",
                    Idioma::Hungaro => "Alapértelmezett · Hangerő",
                    Idioma::Espanol => "Predeterminado · Volumen",
                }
            } else {
                texto(estado.idioma, "volume")
            };

            row.set_subtitle(subtitulo);

            let escala = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 150.0, 1.0);
            escala.set_width_request(120);
            escala.set_draw_value(false);
            escala.set_valign(gtk::Align::Center);
            escala.set_value(node.get("volume").and_then(Value::as_f64).unwrap_or(0.0) * 100.0);
            row.add_suffix(&escala);

            let silencio = gtk::ToggleButton::with_label(texto(estado.idioma, "mute"));
            silencio.set_active(node.get("muted").and_then(Value::as_bool).unwrap_or(false));
            silencio.set_valign(gtk::Align::Center);
            row.add_suffix(&silencio);
            grupo_entrada.add(&row);

            let id_texto = id.to_string();
            let estado_volumen = Rc::clone(&estado);
            escala.connect_value_changed(move |escala| {
                let volumen = format!("{:.0}%", escala.value());
                if let Err(error) = ejecutar_motor(
                    &estado_volumen,
                    &["media", "audio", "volume", &id_texto, &volumen],
                ) {
                    mostrar_error(&estado_volumen, error);
                }
            });

            let id_texto = id.to_string();
            let estado_silencio = Rc::clone(&estado);
            silencio.connect_toggled(move |boton| {
                let valor = if boton.is_active() { "1" } else { "0" };
                if let Err(error) = ejecutar_motor(
                    &estado_silencio,
                    &["media", "audio", "mute", &id_texto, valor],
                ) {
                    mostrar_error(&estado_silencio, error);
                }
            });
        }
    }

    pagina.add(&grupo_entrada);

    let grupo_prueba_mic = adw::PreferencesGroup::new();
    grupo_prueba_mic.set_title(texto_prueba_multimedia(estado.idioma, "mic_test"));

    if !sources.is_empty() {
        let ids = Rc::new(
            sources
                .iter()
                .filter_map(|node| {
                    node.get("id")
                        .and_then(Value::as_u64)
                        .and_then(|id| u32::try_from(id).ok())
                })
                .collect::<Vec<_>>(),
        );

        let etiquetas = sources
            .iter()
            .filter_map(|node| {
                let id = node.get("id").and_then(Value::as_u64)?;
                let nombre = node.get("name").and_then(Value::as_str).unwrap_or("Audio");
                Some(descripcion_nodo_audio(
                    estado.idioma,
                    &audio,
                    "source",
                    id,
                    nombre,
                ))
            })
            .collect::<Vec<_>>();

        let referencias = etiquetas.iter().map(String::as_str).collect::<Vec<_>>();
        let modelo = gtk::StringList::new(&referencias);
        let selector = adw::ComboRow::new();
        selector.set_title(texto_prueba_multimedia(estado.idioma, "mic_pick"));
        selector.set_model(Some(&modelo));

        let predeterminado = audio
            .pointer("/defaults/sourceId")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok());

        if let Some(indice) = ids.iter().position(|id| Some(*id) == predeterminado) {
            selector.set_selected(indice as u32);
        }

        grupo_prueba_mic.add(&selector);

        let fila_nivel = adw::ActionRow::new();
        fila_nivel.set_title(texto_prueba_multimedia(estado.idioma, "mic_level"));
        fila_nivel.set_subtitle(texto_prueba_multimedia(estado.idioma, "mic_level_detail"));

        let nivel = gtk::LevelBar::new();
        nivel.set_min_value(0.0);
        nivel.set_max_value(1.0);
        nivel.set_value(0.0);
        nivel.set_width_request(150);
        nivel.set_valign(gtk::Align::Center);

        let porcentaje = gtk::Label::new(Some(&localizar_visible(idioma_actual(), "0%")));
        porcentaje.set_width_chars(4);
        porcentaje.set_valign(gtk::Align::Center);

        let medir = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "measure"));
        medir.set_valign(gtk::Align::Center);

        fila_nivel.add_suffix(&nivel);
        fila_nivel.add_suffix(&porcentaje);
        fila_nivel.add_suffix(&medir);
        grupo_prueba_mic.add(&fila_nivel);

        let medicion_activa = Rc::new(Cell::new(false));

        {
            let estado_medir = Rc::clone(&estado);
            let selector_medir = selector.clone();
            let ids_medir = Rc::clone(&ids);
            let nivel_medir = nivel.clone();
            let porcentaje_medir = porcentaje.clone();
            let medicion_medir = Rc::clone(&medicion_activa);

            medir.connect_clicked(move |boton| {
                if medicion_medir.get() {
                    medicion_medir.set(false);
                    boton.set_label(texto_prueba_multimedia(estado_medir.idioma, "measure"));
                    return;
                }

                let Some(id) = ids_medir.get(selector_medir.selected() as usize).copied() else {
                    return;
                };

                medicion_medir.set(true);
                boton.set_label(texto_prueba_multimedia(estado_medir.idioma, "measure_stop"));

                let resultado = medir_microfono_gui(
                    &estado_medir,
                    id,
                    &nivel_medir,
                    &porcentaje_medir,
                    Rc::clone(&medicion_medir),
                );

                medicion_medir.set(false);
                boton.set_label(texto_prueba_multimedia(estado_medir.idioma, "measure"));

                if let Err(error) = resultado {
                    mostrar_error(&estado_medir, error);
                }
            });
        }

        let fila_voz = adw::ActionRow::new();
        fila_voz.set_title(texto_prueba_multimedia(estado.idioma, "mic_test"));
        fila_voz.set_subtitle(texto_prueba_multimedia(estado.idioma, "voice_detail"));

        let botones = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        botones.set_valign(gtk::Align::Center);
        botones.set_halign(gtk::Align::End);
        botones.set_hexpand(false);
        let grabar = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "record"));
        grabar.add_css_class("suggested-action");
        grabar.set_valign(gtk::Align::Center);
        let reproducir = gtk::Button::with_label(texto_prueba_multimedia(estado.idioma, "play"));
        reproducir.set_sensitive(false);
        reproducir.set_valign(gtk::Align::Center);

        botones.append(&grabar);
        botones.append(&reproducir);
        fila_voz.add_suffix(&botones);
        grupo_prueba_mic.add(&fila_voz);

        let muestra = Rc::new(RefCell::new(None::<String>));

        {
            let estado_grabar = Rc::clone(&estado);
            let selector_grabar = selector.clone();
            let ids_grabar = Rc::clone(&ids);
            let muestra_grabar = Rc::clone(&muestra);
            let reproducir_grabar = reproducir.clone();

            grabar.connect_clicked(move |boton| {
                let Some(id) = ids_grabar.get(selector_grabar.selected() as usize).copied() else {
                    return;
                };

                let etiqueta_normal = texto_prueba_multimedia(estado_grabar.idioma, "record");
                boton.set_label(texto_prueba_multimedia(
                    estado_grabar.idioma,
                    "recording_button",
                ));
                boton.set_sensitive(false);

                muestra_grabar.borrow_mut().take();
                reproducir_grabar.set_sensitive(false);

                let id = id.to_string();
                match ejecutar_json(
                    &estado_grabar,
                    &[
                        "media",
                        "mic",
                        "record",
                        &id,
                        "--seconds",
                        "3",
                        "--yes",
                        "--json",
                    ],
                ) {
                    Ok(resultado) => {
                        let Some(path) = resultado
                            .get("path")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                        else {
                            mostrar_error(
                                &estado_grabar,
                                "Korunix no devolvió la grabación temporal.".to_string(),
                            );
                            return;
                        };

                        *muestra_grabar.borrow_mut() = Some(path);
                        reproducir_grabar.set_sensitive(true);
                        boton.set_label(texto_prueba_multimedia(
                            estado_grabar.idioma,
                            "record_again",
                        ));
                        mostrar_exito(
                            &estado_grabar,
                            texto_prueba_multimedia(estado_grabar.idioma, "recorded"),
                        );
                    }
                    Err(error) => {
                        boton.set_label(etiqueta_normal);
                        mostrar_error(&estado_grabar, error);
                    }
                }
                boton.set_sensitive(true);
            });
        }

        {
            let estado_reproducir = Rc::clone(&estado);
            let muestra_reproducir = Rc::clone(&muestra);
            let reproducir_boton = reproducir.clone();

            reproducir.connect_clicked(move |boton| {
                let Some(path) = muestra_reproducir.borrow().clone() else {
                    reproducir_boton.set_sensitive(false);
                    return;
                };

                boton.set_label(texto_prueba_multimedia(
                    estado_reproducir.idioma,
                    "playing_button",
                ));
                boton.set_sensitive(false);

                match ejecutar_json(
                    &estado_reproducir,
                    &["media", "mic", "play", &path, "--delete", "--yes", "--json"],
                ) {
                    Ok(_) => {
                        boton.set_label(texto_prueba_multimedia(estado_reproducir.idioma, "play"));
                        muestra_reproducir.borrow_mut().take();
                        reproducir_boton.set_sensitive(false);
                        mostrar_exito(
                            &estado_reproducir,
                            texto_prueba_multimedia(estado_reproducir.idioma, "played"),
                        );
                    }
                    Err(error) => {
                        boton.set_label(texto_prueba_multimedia(estado_reproducir.idioma, "play"));
                        boton.set_sensitive(true);
                        mostrar_error(&estado_reproducir, error);
                    }
                }
            });
        }
    }

    pagina.add(&grupo_prueba_mic);

    let grupo_camaras = adw::PreferencesGroup::new();
    grupo_camaras.set_title(texto(estado.idioma, "cameras"));

    let camaras = datos
        .pointer("/cameras/devices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut vistas = Vec::<String>::new();

    for camara in camaras {
        let titulo = camara
            .get("card")
            .and_then(Value::as_str)
            .filter(|valor| !valor.is_empty())
            .unwrap_or("Cámara")
            .to_string();

        if vistas.iter().any(|actual| actual == &titulo) {
            continue;
        }

        vistas.push(titulo.clone());

        let bus = camara
            .get("bus")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();

        let virtual_device = camara
            .get("virtual")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let disponible = camara
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let base = match (virtual_device, disponible, bus.contains("usb")) {
            (true, true, _) => match estado.idioma {
                Idioma::Ingles => "Virtual · Available",
                Idioma::BelarusLatino => "Virtuałny · Dastupny",
                Idioma::Belarus => "Віртуальны · Даступны",
                Idioma::Catalan => "Virtual · Disponible",
                Idioma::Checo => "Virtuální · Dostupné",
                Idioma::Aleman => "Virtuell · Verfügbar",
                Idioma::Frances => "Virtuel · Disponible",
                Idioma::Gallego => "Virtual · Dispoñible",
                Idioma::Italiano => "Virtuale · Disponibile",
                Idioma::Coreano => "가상 · 사용 가능",
                Idioma::Kurdo => "Virtual · Berdest",
                Idioma::Neerlandes => "Virtueel · Beschikbaar",
                Idioma::NoruegoNynorsk => "Virtuelt · Tilgjengelig",
                Idioma::Polaco => "Wirtualny · Dostępny",
                Idioma::PortuguesBrasil => "Virtual · Disponível",
                Idioma::Ruso => "Виртуальный · Доступный",
                Idioma::Sueco => "Virtuell · Tillgänglig",
                Idioma::Turco => "Sanal · Mevcut",
                Idioma::Ucraniano => "Віртуальний · Доступний",
                Idioma::Vietnamita => "Ảo · Có sẵn",
                Idioma::ChinoSimplificado => "虚拟 · 可用",
                Idioma::Hungaro => "Virtuális · Elérhető",
                Idioma::Espanol => "Virtual · Disponible",
            },
            (true, false, _) => match estado.idioma {
                Idioma::Ingles => "Virtual · Waiting for a video source",
                Idioma::BelarusLatino => "Virtuałny · Čakannje krynicy videa",
                Idioma::Belarus => "Віртуальны · Чаканне крыніцы відэа",
                Idioma::Catalan => "Virtual · Esperant una font de vídeo",
                Idioma::Checo => "Virtuální · Čekání na zdroj videa",
                Idioma::Aleman => "Virtuell · Warten auf eine Videoquelle",
                Idioma::Frances => "Virtuel · En attente d'une source vidéo",
                Idioma::Gallego => "Virtual · Agardando por unha fonte de vídeo",
                Idioma::Italiano => "Virtuale · In attesa di una sorgente video",
                Idioma::Coreano => "가상 · 비디오 소스를 기다리는 중",
                Idioma::Kurdo => "Virtual · Li benda çavkaniyek vîdyoyê ne",
                Idioma::Neerlandes => "Virtueel · Wachten op een videobron",
                Idioma::NoruegoNynorsk => "Virtual · Venter på en videokilde",
                Idioma::Polaco => "Wirtualny · Oczekiwanie na źródło wideo",
                Idioma::PortuguesBrasil => "Virtual · Aguardando uma fonte de vídeo",
                Idioma::Ruso => "Виртуальный · Ожидание источника видео",
                Idioma::Sueco => "Virtuell · Väntar på en videokälla",
                Idioma::Turco => "Sanal · Bir video kaynağı bekleniyor",
                Idioma::Ucraniano => "Віртуальний · Очікування джерела відео",
                Idioma::Vietnamita => "Ảo · Đang chờ nguồn video",
                Idioma::ChinoSimplificado => "虚拟·等待视频源",
                Idioma::Hungaro => "Virtuális · Videóforrásra vár",
                Idioma::Espanol => "Virtual · Esperando una fuente de vídeo",
            },
            (false, true, true) => match estado.idioma {
                Idioma::Ingles => "USB · Available",
                Idioma::BelarusLatino => "USB · Dastupny",
                Idioma::Belarus => "USB · Даступны",
                Idioma::Catalan => "USB · Disponible",
                Idioma::Checo => "USB · K dispozici",
                Idioma::Aleman => "USB · Verfügbar",
                Idioma::Frances => "USB · Disponible",
                Idioma::Gallego => "USB · Dispoñible",
                Idioma::Italiano => "USB · Disponibile",
                Idioma::Coreano => "USB · 사용 가능",
                Idioma::Kurdo => "USB · Berdest e",
                Idioma::Neerlandes => "USB · Beschikbaar",
                Idioma::NoruegoNynorsk => "USB · Tilgjengelig",
                Idioma::Polaco => "USB · Dostępne",
                Idioma::PortuguesBrasil => "USB · Disponível",
                Idioma::Ruso => "USB · Доступен",
                Idioma::Sueco => "USB · Tillgänglig",
                Idioma::Turco => "USB · Mevcut",
                Idioma::Ucraniano => "USB · Є",
                Idioma::Vietnamita => "USB · Có sẵn",
                Idioma::ChinoSimplificado => "USB·可用",
                Idioma::Hungaro => "USB · Elérhető",
                Idioma::Espanol => "USB · Disponible",
            },
            (false, true, false) => match estado.idioma {
                Idioma::Ingles => "Available",
                Idioma::BelarusLatino => "Dastupny",
                Idioma::Belarus => "Даступна",
                Idioma::Catalan => "Disponible",
                Idioma::Checo => "Dostupná",
                Idioma::Aleman => "Verfügbar",
                Idioma::Frances => "Disponible",
                Idioma::Gallego => "Dispoñible",
                Idioma::Italiano => "Disponibili",
                Idioma::Coreano => "사용 가능",
                Idioma::Kurdo => "Heyî",
                Idioma::Neerlandes => "Beschikbaar",
                Idioma::NoruegoNynorsk => "Tilgjengeleg(e)",
                Idioma::Polaco => "Dostępne",
                Idioma::PortuguesBrasil => "Disponível",
                Idioma::Ruso => "Доступные",
                Idioma::Sueco => "Tillgänglig",
                Idioma::Turco => "Kullanılabilir",
                Idioma::Ucraniano => "Доступні пристрої",
                Idioma::Vietnamita => "Khả dụng",
                Idioma::ChinoSimplificado => "可用设备",
                Idioma::Hungaro => "Elérhető",
                Idioma::Espanol => "Disponible",
            },
            _ => match estado.idioma {
                Idioma::Ingles => "Unavailable",
                Idioma::BelarusLatino => "Njedastupny",
                Idioma::Belarus => "Недаступна",
                Idioma::Catalan => "No disponible",
                Idioma::Checo => "Nedostupné",
                Idioma::Aleman => "Nicht verfügbar",
                Idioma::Frances => "Indisponible",
                Idioma::Gallego => "Non dispoñible",
                Idioma::Italiano => "Non disponibile",
                Idioma::Coreano => "사용 불가",
                Idioma::Kurdo => "Unavailable",
                Idioma::Neerlandes => "Niet beschikbaar",
                Idioma::NoruegoNynorsk => "Utilgjengeleg",
                Idioma::Polaco => "Niedostępne",
                Idioma::PortuguesBrasil => "Indisponível",
                Idioma::Ruso => "Недоступно",
                Idioma::Sueco => "Otillgänglig",
                Idioma::Turco => "Mevcut değil",
                Idioma::Ucraniano => "Недоступно",
                Idioma::Vietnamita => "Không khả dụng",
                Idioma::ChinoSimplificado => "不可用",
                Idioma::Hungaro => "Nem érhető el",
                Idioma::Espanol => "No disponible",
            },
        };

        let modos = resumen_modos_camara(&camara);
        let detalle = if modos.is_empty() {
            base.to_string()
        } else {
            format!("{base} · {modos}")
        };

        grupo_camaras.add(&fila(&titulo, &detalle));
    }

    if vistas.is_empty() {
        grupo_camaras.add(&fila(
            texto(estado.idioma, "status"),
            texto(estado.idioma, "empty"),
        ));
    }

    pagina.add(&grupo_camaras);

    let grupo_prueba_camara = adw::PreferencesGroup::new();
    grupo_prueba_camara.set_title(texto_prueba_camara(estado.idioma, "title"));

    let camaras_prueba = datos
        .pointer("/cameras/devices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut camaras_vistas = Vec::<String>::new();

    for camara in camaras_prueba {
        let titulo = camara
            .get("card")
            .and_then(Value::as_str)
            .filter(|valor| !valor.is_empty())
            .unwrap_or("Cámara")
            .to_string();
        if camaras_vistas.iter().any(|actual| actual == &titulo) {
            continue;
        }
        camaras_vistas.push(titulo.clone());

        let Some(device) = camara
            .get("device")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };

        let row = adw::ActionRow::new();
        row.set_title(&titulo);

        let disponibles = resumen_modos_camara(&camara);
        let disponible = camara
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let prueba = match estado.idioma {
            Idioma::Ingles => "Test: 640×360 · automatic FPS",
            Idioma::BelarusLatino => "Test: 640×360 · aŭtamatyčny FPS",
            Idioma::Belarus => "Тэст: 640×360 · аўтаматычны FPS",
            Idioma::Catalan => "Prova: 640×360 · FPS automàtic",
            Idioma::Checo => "Test: 640×360 · automatické FPS",
            Idioma::Aleman => "Test: 640×360 · automatische FPS",
            Idioma::Frances => "Test : 640×360 · FPS automatique",
            Idioma::Gallego => "Proba: 640×360 · FPS automático",
            Idioma::Italiano => "Test: 640×360 · FPS automatico",
            Idioma::Coreano => "테스트: 640×360 · 자동 FPS",
            Idioma::Kurdo => "Test: 640×360 · FPS otomatîk",
            Idioma::Neerlandes => "Test: 640×360 · automatische FPS",
            Idioma::NoruegoNynorsk => "Test: 640×360 · automatisk FPS",
            Idioma::Polaco => "Test: 640×360 · automatyczny FPS",
            Idioma::PortuguesBrasil => "Teste: 640×360 · FPS automático",
            Idioma::Ruso => "Тест: 640×360 · автоматический FPS",
            Idioma::Sueco => "Test: 640×360 · automatisk FPS",
            Idioma::Turco => "Test: 640×360 · otomatik FPS",
            Idioma::Ucraniano => "Тест: 640×360 · автоматичний FPS",
            Idioma::Vietnamita => "Kiểm tra: 640×360 · FPS tự động",
            Idioma::ChinoSimplificado => "测试：640×360·自动FPS",
            Idioma::Hungaro => "Próba: 640×360 · automatikus FPS",
            Idioma::Espanol => "Prueba: 640×360 · FPS automático",
        };

        let subtitulo = if !disponible {
            match estado.idioma {
                Idioma::Ingles => {
                    "Waiting for a video source. The test will become available automatically."
                        .to_string()
                }
                Idioma::BelarusLatino => {
                    "Čakannje krynicy videa. Test stanje dastupnym aŭtamatyčna.".to_string()
                }
                Idioma::Belarus => {
                    "Чаканне крыніцы відэа. Тэст стане даступным аўтаматычна.".to_string()
                }
                Idioma::Catalan => {
                    "Esperant una font de vídeo. La prova estarà disponible automàticament."
                        .to_string()
                }
                Idioma::Checo => {
                    "Čekání na zdroj videa. Test bude k dispozici automaticky.".to_string()
                }
                Idioma::Aleman => {
                    "Warten auf eine Videoquelle. Der Test wird automatisch verfügbar sein."
                        .to_string()
                }
                Idioma::Frances => {
                    "En attente d'une source vidéo. Le test deviendra disponible automatiquement."
                        .to_string()
                }
                Idioma::Gallego => {
                    "Agardando por unha fonte de vídeo. A proba estará dispoñible automaticamente."
                        .to_string()
                }
                Idioma::Italiano => {
                    "In attesa di una sorgente video. Il test sarà disponibile automaticamente."
                        .to_string()
                }
                Idioma::Coreano => {
                    "비디오 소스를 기다리고 있습니다. 테스트가 자동으로 제공됩니다.".to_string()
                }
                Idioma::Kurdo => {
                    "Li benda çavkaniyek vîdyoyê ne. Test dê bixweber peyda bibe.".to_string()
                }
                Idioma::Neerlandes => {
                    "Wachten op een videobron. De test wordt automatisch beschikbaar.".to_string()
                }
                Idioma::NoruegoNynorsk => {
                    "Venter på en videokilde. Testen blir automatisk tilgjengelig.".to_string()
                }
                Idioma::Polaco => {
                    "Czekam na źródło wideo. Test będzie dostępny automatycznie.".to_string()
                }
                Idioma::PortuguesBrasil => {
                    "Aguardando uma fonte de vídeo. O teste ficará disponível automaticamente."
                        .to_string()
                }
                Idioma::Ruso => {
                    "Жду источник видео. Тест станет доступен автоматически.".to_string()
                }
                Idioma::Sueco => {
                    "Väntar på en videokälla. Testet blir automatiskt tillgängligt.".to_string()
                }
                Idioma::Turco => {
                    "Video kaynağı bekleniyor. Test otomatik olarak kullanılabilir hale gelecektir."
                        .to_string()
                }
                Idioma::Ucraniano => {
                    "Очікування джерела відео. Тест стане доступним автоматично.".to_string()
                }
                Idioma::Vietnamita => {
                    "Đang chờ nguồn video. Bài kiểm tra sẽ tự động có sẵn.".to_string()
                }
                Idioma::ChinoSimplificado => "等待视频源。测试将自动变为可用。".to_string(),
                Idioma::Hungaro => {
                    "Videóforrásra vár. A próba automatikusan elérhetővé válik.".to_string()
                }
                Idioma::Espanol => {
                    "Esperando una fuente de vídeo. La prueba se habilitará automáticamente."
                        .to_string()
                }
            }
        } else if disponibles.is_empty() {
            format!(
                "{} · {prueba}",
                texto_prueba_camara(estado.idioma, "detail")
            )
        } else {
            format!(
                "{} · {prueba} · {}: {disponibles}",
                texto_prueba_camara(estado.idioma, "detail"),
                match estado.idioma {
                    Idioma::Ingles => "Available",
                    Idioma::BelarusLatino => "Dastupny",
                    Idioma::Belarus => "Даступна",
                    Idioma::Catalan => "Disponible",
                    Idioma::Checo => "Dostupná",
                    Idioma::Aleman => "Verfügbar",
                    Idioma::Frances => "Disponible",
                    Idioma::Gallego => "Dispoñible",
                    Idioma::Italiano => "Disponibili",
                    Idioma::Coreano => "사용 가능",
                    Idioma::Kurdo => "Heyî",
                    Idioma::Neerlandes => "Beschikbaar",
                    Idioma::NoruegoNynorsk => "Tilgjengeleg(e)",
                    Idioma::Polaco => "Dostępne",
                    Idioma::PortuguesBrasil => "Disponível",
                    Idioma::Ruso => "Доступные",
                    Idioma::Sueco => "Tillgänglig",
                    Idioma::Turco => "Kullanılabilir",
                    Idioma::Ucraniano => "Доступні пристрої",
                    Idioma::Vietnamita => "Khả dụng",
                    Idioma::ChinoSimplificado => "可用设备",
                    Idioma::Hungaro => "Elérhető",
                    Idioma::Espanol => "Disponibles",
                },
            )
        };

        row.set_subtitle(&subtitulo);

        let boton = gtk::Button::with_label(texto_prueba_camara(estado.idioma, "action"));
        boton.set_sensitive(disponible);
        boton.add_css_class("suggested-action");
        boton.set_valign(gtk::Align::Center);
        row.add_suffix(&boton);
        grupo_prueba_camara.add(&row);

        let estado_camara = Rc::clone(&estado);
        let titulo_camara = titulo.clone();
        boton.connect_clicked(move |_| {
            abrir_prueba_camara_gui(
                Rc::clone(&estado_camara),
                device.clone(),
                titulo_camara.clone(),
            );
        });
    }

    pagina.add(&grupo_prueba_camara);
    pagina
}

fn cuerpo_confirmacion_canal(idioma: Idioma, destino: &str) -> String {
    let canal = texto(
        idioma,
        if destino == "unstable" {
            "unstable"
        } else {
            "stable"
        },
    );

    match idioma {
        Idioma::Ingles => format!(
            "Korunix evaluated the change to “{canal}” without modifying the configuration. If you continue, only the channel declaration will be changed."
        ),
        Idioma::BelarusLatino => format!(
            "Korunix acaniŭ zmjanjennje ŭ «{canal}» bjez zmjeny kanfihuracyi. Kałi vy pracjahnjecje, budzje zmjenjena tołki dekłaracyja kanała."
        ),
        Idioma::Belarus => format!(
            "Korunix ацаніў змяненне ў «{canal}» без змены канфігурацыі. Калі вы працягнеце, будзе зменена толькі дэкларацыя канала."
        ),
        Idioma::Catalan => format!(
            "Korunix va avaluar el canvi a \"{canal}\" sense modificar la configuració. Si continueu, només es canviarà la declaració del canal."
        ),
        Idioma::Checo => format!(
            "Korunix vyhodnotil změnu na „{canal}“ bez úpravy konfigurace. Pokud budete pokračovat, změní se pouze deklarace kanálu."
        ),
        Idioma::Aleman => format!(
            "Korunix hat die Änderung zu „{canal}“ ausgewertet, ohne die Konfiguration zu ändern. Wenn Sie fortfahren, wird nur die Kanaldeklaration geändert."
        ),
        Idioma::Frances => format!(
            "Korunix a évalué le changement apporté à « {canal} » sans modifier la configuration. Si vous continuez, seule la déclaration du canal sera modifiée."
        ),
        Idioma::Gallego => format!(
            "Korunix avaliou o cambio a \"{canal}\" sen modificar a configuración. Se continúas, só se cambiará a declaración da canle."
        ),
        Idioma::Italiano => format!(
            "Korunix ha valutato la modifica a “{canal}” senza modificare la configurazione. Se continui, verrà modificata solo la dichiarazione del canale."
        ),
        Idioma::Coreano => format!(
            "Korunix는 구성을 수정하지 않고 “{canal}”에 대한 변경 사항을 평가했습니다. 계속하면 채널 선언만 변경됩니다."
        ),
        Idioma::Kurdo => format!(
            "Korunix guherîna \"{canal}\" nirxand bêyî ku veavakirinê biguherîne. Heke hûn berdewam bikin, tenê daxuyaniya kanalê dê were guhertin."
        ),
        Idioma::Neerlandes => format!(
            "Korunix heeft de wijziging in “{canal}” geëvalueerd zonder de configuratie te wijzigen. Als u doorgaat, wordt alleen de kanaaldeclaratie gewijzigd."
        ),
        Idioma::NoruegoNynorsk => format!(
            "Korunix evaluerte endringen til \"{canal}\" uten å endre konfigurasjonen. Hvis du fortsetter, vil kun kanalerklæringen endres."
        ),
        Idioma::Polaco => format!(
            "Korunix ocenił zmianę na „{canal}” bez modyfikowania konfiguracji. Jeśli będziesz kontynuować, zmieniona zostanie tylko deklaracja kanału."
        ),
        Idioma::PortuguesBrasil => format!(
            "Korunix avaliou a mudança para “{canal}” sem modificar a configuração. Se você continuar, apenas a declaração do canal será alterada."
        ),
        Idioma::Ruso => format!(
            "Korunix оценил изменение «{canal}» без изменения конфигурации. Если вы продолжите, будет изменено только объявление канала."
        ),
        Idioma::Sueco => format!(
            "Korunix utvärderade ändringen till \"{canal}\" utan att ändra konfigurationen. Om du fortsätter kommer endast kanaldeklarationen att ändras."
        ),
        Idioma::Turco => format!(
            "Korunix, yapılandırmayı değiştirmeden “{canal}” değişikliğini değerlendirdi. Devam ederseniz yalnızca kanal bildirimi değişecektir."
        ),
        Idioma::Ucraniano => format!(
            "Корунікс оцінив зміну до «{canal}» без зміни конфігурації. Якщо ви продовжите, буде змінено лише оголошення каналу."
        ),
        Idioma::Vietnamita => format!(
            "Korunix đã đánh giá sự thay đổi thành “{canal}” mà không sửa đổi cấu hình. Nếu bạn tiếp tục, chỉ khai báo kênh sẽ được thay đổi."
        ),
        Idioma::ChinoSimplificado => format!(
            "Korunix 在不修改配置的情况下评估了对“{canal}”的更改。如果继续，则只会更改通道声明。"
        ),
        Idioma::Hungaro => format!(
            "A Korunix a(z) „{canal}” csatornára váltást a konfiguráció módosítása nélkül ellenőrizte. Folytatáskor csak a csatorna deklarációja változik meg."
        ),
        Idioma::Espanol => format!(
            "Korunix evaluó el cambio a «{canal}» sin modificar la configuración. Si continúas, se cambiará únicamente la declaración del canal."
        ),
    }
}

fn reemplazar_pagina(
    stack: &gtk::Stack,
    nombre: &str,
    titulo: &str,
    pagina: &adw::PreferencesPage,
) {
    if let Some(anterior) = stack.child_by_name(nombre) {
        stack.remove(&anterior);
    }

    stack.add_titled(pagina, Some(nombre), titulo);
}

fn ejecutar_json_owned(estado: &Estado, argumentos: &[String]) -> Result<Value, String> {
    let referencias = argumentos.iter().map(String::as_str).collect::<Vec<_>>();
    ejecutar_json(estado, &referencias)
}

fn aplicar_configuracion_gui(estado: &Estado) -> Result<Value, String> {
    ejecutar_json(estado, &["preview", "--json"])?;
    ejecutar_json(estado, &["apply", "--yes", "--json"])
}

fn ejecutar_json_con_stdin(
    estado: &Estado,
    argumentos: &[&str],
    entrada: &str,
) -> Result<Value, String> {
    let mut hijo = Command::new(&estado.motor)
        .args(argumentos)
        .current_dir(&estado.raiz)
        .env("KORUNIX_ROOT", &estado.raiz)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("No pude iniciar el motor: {error}"))?;

    let Some(mut stdin) = hijo.stdin.take() else {
        let _ = hijo.kill();
        let _ = hijo.wait();
        return Err("No pude entregar la contraseña de forma protegida.".to_string());
    };

    std::io::Write::write_all(&mut stdin, entrada.as_bytes())
        .map_err(|error| format!("No pude entregar la contraseña: {error}"))?;
    drop(stdin);

    let salida = hijo
        .wait_with_output()
        .map_err(|error| format!("No pude esperar al motor: {error}"))?;

    if !salida.status.success() {
        let error = String::from_utf8_lossy(&salida.stderr).trim().to_string();
        return Err(if error.is_empty() {
            "La operación protegida terminó con error.".to_string()
        } else {
            error
        });
    }

    let texto = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    serde_json::from_str(&texto)
        .map_err(|error| format!("El motor devolvió JSON inválido: {error}"))
}

fn dialogo_accion(
    boton: &gtk::Button,
    estado: &Estado,
    cuerpo: &str,
    accion: &str,
    destructiva: bool,
) -> adw::MessageDialog {
    dialogo_confirmacion(boton, estado.idioma, cuerpo, accion, destructiva)
}

fn indice(valor: &str, opciones: &[String]) -> u32 {
    opciones
        .iter()
        .position(|actual| actual == valor)
        .unwrap_or(0) as u32
}

fn terminos_busqueda_pagina(nombre: &str) -> &'static str {
    match nombre {
        "summary" => {
            "resumen inicio estado configuración configuracion equipo ordenador computadora sistema"
        }
        "updates" => {
            "actualizaciones actualizar sistema software versiones canal estable inestable update"
        }
        "hardware" => {
            "hardware equipo ordenador computadora procesador cpu memoria ram gráficos graficos gpu red controlador"
        }
        "media" => {
            "sonido audio altavoces auriculares micrófono microfono cámara camara webcam vídeo video"
        }
        "storage" => {
            "almacenamiento disco discos unidad unidades usb expulsar extraíble extraible guardar datos"
        }
        "firmware" => {
            "firmware dispositivos actualizaciones bios uefi placa hardware"
        }
        "applications" => {
            "aplicaciones programas software navegador browser firefox chrome correo thunderbird editor texto kwrite kate oficina juegos diseño multimedia instalar eliminar"
        }
        "appearance" => {
            "apariencia tema claro oscuro automático automatico everforest escritorio niri hyprland plasma cinnamon fondo iconos"
        }
        "localization" => {
            "idioma región region país pais formatos zona horaria hora teclado distribución distribucion variante entrada"
        }
        "people" => {
            "personas persona usuario usuarios cuenta cuentas contraseña contrasena avatar administrador estándar estandar"
        }
        "backups" => {
            "copias copia seguridad exportar restaurar restauración restauracion historial backup"
        }
        "maintenance" => {
            "mantenimiento limpiar limpieza recuperación recuperacion versiones reinicio rollback"
        }
        _ => "",
    }
}

fn nombre_desde_id(id: &str) -> String {
    id.split(|c| c == '-' || c == '_')
        .filter(|parte| !parte.is_empty())
        .map(|parte| {
            let mut caracteres = parte.chars();
            match caracteres.next() {
                Some(primero) => primero.to_uppercase().collect::<String>() + caracteres.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PresentacionAplicacion {
    nombre: String,
    descripcion: String,
    categoria: String,
}

fn presentacion_aplicacion(datos: &Value, id: &str) -> Option<PresentacionAplicacion> {
    let entrada = datos.get("presentation")?.get(id)?;

    Some(PresentacionAplicacion {
        nombre: entrada.get("name")?.as_str()?.to_string(),
        descripcion: entrada.get("description")?.as_str()?.to_string(),
        categoria: entrada.get("category")?.as_str()?.to_string(),
    })
}

fn ids_aplicaciones_seleccionadas(datos: &Value) -> Vec<String> {
    datos
        .get("selected")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|valor| valor.as_str().map(str::to_string))
        .collect()
}

fn escritorio_usa_noctalia(datos: &Value) -> bool {
    let principal = datos
        .pointer("/desktop/primary")
        .and_then(Value::as_str)
        .unwrap_or("");

    if matches!(principal, "niri" | "hyprland") {
        return true;
    }

    datos
        .pointer("/desktop/additional")
        .and_then(Value::as_array)
        .map(|adicionales| {
            adicionales
                .iter()
                .any(|valor| matches!(valor.as_str(), Some("niri" | "hyprland")))
        })
        .unwrap_or(false)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnidadActualizacion {
    titulo: String,
    detalle: String,
    targets: Vec<String>,
}

fn targets_presentes(objetivos: &[String], ids: &[&str]) -> Vec<String> {
    ids.iter()
        .filter(|id| objetivos.iter().any(|objetivo| objetivo.as_str() == **id))
        .map(|id| (*id).to_string())
        .collect()
}

fn unidades_actualizacion_humanas(
    objetivos: &[String],
    aplicaciones: &[String],
    noctalia_relevante: bool,
) -> Vec<UnidadActualizacion> {
    let mut unidades = Vec::<UnidadActualizacion>::new();
    let mut usados = Vec::<String>::new();

    let juegos_instalados = aplicaciones
        .iter()
        .any(|id| matches!(id.as_str(), "genshin-impact" | "honkai-star-rail"));
    let juegos = targets_presentes(objetivos, &["aagl", "aaglStable"]);
    if juegos_instalados && !juegos.is_empty() {
        usados.extend(juegos.iter().cloned());
        unidades.push(UnidadActualizacion {
            titulo: "Juegos instalados".to_string(),
            detalle: "Actualiza los launchers y la compatibilidad que necesitan los juegos de anime instalados.".to_string(),
            targets: juegos,
        });
    }

    let spotify = targets_presentes(objetivos, &["spicetify-nix"]);
    if aplicaciones.iter().any(|id| id == "spotify") && !spotify.is_empty() {
        usados.extend(spotify.iter().cloned());
        unidades.push(UnidadActualizacion {
            titulo: "Spotify".to_string(),
            detalle:
                "Actualiza la integración funcional y visual que Korunix mantiene para Spotify."
                    .to_string(),
            targets: spotify,
        });
    }

    let steam = targets_presentes(objetivos, &["millennium"]);
    if aplicaciones.iter().any(|id| id == "steam") && !steam.is_empty() {
        usados.extend(steam.iter().cloned());
        unidades.push(UnidadActualizacion {
            titulo: "Steam".to_string(),
            detalle: "Actualiza la integración de Steam administrada junto con su instalación."
                .to_string(),
            targets: steam,
        });
    }

    let noctalia = targets_presentes(objetivos, &["noctalia"]);
    if noctalia_relevante && !noctalia.is_empty() {
        usados.extend(noctalia.iter().cloned());
        unidades.push(UnidadActualizacion {
            titulo: "Noctalia".to_string(),
            detalle: "Actualiza el panel y las integraciones utilizadas por Niri y Hyprland."
                .to_string(),
            targets: noctalia,
        });
    }

    let sistema = objetivos
        .iter()
        .filter(|id| !usados.iter().any(|usado| usado == *id))
        .cloned()
        .collect::<Vec<_>>();

    if !sistema.is_empty() {
        unidades.insert(
            0,
            UnidadActualizacion {
                titulo: "Sistema y aplicaciones".to_string(),
                detalle: "Actualiza NixOS y las aplicaciones que comparten su base. Korunix incluye automáticamente las piezas de compatibilidad necesarias.".to_string(),
                targets: sistema,
            },
        );
    }

    unidades
}

fn targets_unidades_seleccionadas(unidades: &[(Vec<String>, gtk::CheckButton)]) -> Vec<String> {
    let mut resultado = unidades
        .iter()
        .filter(|(_, check)| check.is_active())
        .flat_map(|(targets, _)| targets.iter().cloned())
        .collect::<Vec<_>>();

    resultado.sort();
    resultado.dedup();
    resultado
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultadoAplicacionExterna {
    id: String,
    fuente: String,
    nombre: String,
    descripcion: String,
}

fn resultados_aplicaciones_externas(
    datos: &Value,
    fuente: &str,
) -> Vec<ResultadoAplicacionExterna> {
    let mut resultados = Vec::new();

    if let Some(items) = datos.get("results").and_then(Value::as_array) {
        for item in items.iter().take(40) {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };

            let nombre = item
                .get("name")
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty())
                .unwrap_or(id);

            let descripcion = item
                .get("description")
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("Instala {nombre} como aplicación adicional."));

            resultados.push(ResultadoAplicacionExterna {
                id: id.to_string(),
                fuente: fuente.to_string(),
                nombre: nombre.to_string(),
                descripcion,
            });
        }
    }

    if let Some(items) = datos.get("results").and_then(Value::as_object) {
        for (id, item) in items.iter().take(40) {
            let nombre = item
                .get("pname")
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    id.rsplit('.')
                        .next()
                        .unwrap_or(id)
                        .replace('-', " ")
                        .replace('_', " ")
                });

            let descripcion = item
                .get("description")
                .and_then(Value::as_str)
                .filter(|valor| !valor.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("Instala {nombre} como aplicación adicional."));

            resultados.push(ResultadoAplicacionExterna {
                id: id.clone(),
                fuente: fuente.to_string(),
                nombre,
                descripcion,
            });
        }
    }

    resultados
}

fn nombre_escritorio_humano(id: &str) -> String {
    match id {
        "niri" => "Niri".to_string(),
        "hyprland" => "Hyprland".to_string(),
        "plasma" => "KDE Plasma".to_string(),
        "cinnamon" => "Cinnamon".to_string(),
        _ => nombre_desde_id(id),
    }
}

fn variante_teclado_humana(valor: &str) -> String {
    match valor {
        "" => "Predeterminada".to_string(),
        "deadtilde" => "Tildes mediante tecla muerta".to_string(),
        "nodeadkeys" => "Sin teclas muertas".to_string(),
        _ => nombre_desde_id(valor),
    }
}

fn fila_aplicacion(
    estado: Rc<Estado>,
    id: String,
    fuente: String,
    nombre: String,
    descripcion: String,
    activa: bool,
) -> adw::ActionRow {
    let fila = adw::ActionRow::new();
    fila.set_title(&nombre);
    fila.set_subtitle(&descripcion);

    let boton = gtk::Button::with_label(texto(
        estado.idioma,
        if activa { "remove" } else { "install" },
    ));
    boton.set_valign(gtk::Align::Center);
    if !activa {
        boton.add_css_class("suggested-action");
    }
    fila.add_suffix(&boton);

    let estado_click = Rc::clone(&estado);
    boton.connect_clicked(move |boton| {
        let accion = if activa { "off" } else { "on" };
        let plan = vec![
            "applications".to_string(),
            "set".to_string(),
            id.clone(),
            accion.to_string(),
            "--source".to_string(),
            fuente.clone(),
            "--plan".to_string(),
            "--json".to_string(),
        ];

        if let Err(error) = ejecutar_json_owned(&estado_click, &plan) {
            mostrar_error(&estado_click, error);
            return;
        }

        let cuerpo = if activa {
            format!(
                "¿Eliminar «{nombre}»? Korunix retirará esta aplicación de la configuración del equipo."
            )
        } else {
            format!("¿Instalar «{nombre}»? {descripcion}")
        };

        let dialogo = dialogo_accion(
            boton,
            &estado_click,
            &cuerpo,
            texto(
                estado_click.idioma,
                if activa { "remove" } else { "install" },
            ),
            activa,
        );

        let estado_aplicar = Rc::clone(&estado_click);
        let id_aplicar = id.clone();
        let fuente_aplicar = fuente.clone();

        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let ejecutar = vec![
                "applications".to_string(),
                "set".to_string(),
                id_aplicar.clone(),
                accion.to_string(),
                "--source".to_string(),
                fuente_aplicar.clone(),
                "--yes".to_string(),
                "--json".to_string(),
            ];

            match ejecutar_json_owned(&estado_aplicar, &ejecutar)
                .and_then(|_| aplicar_configuracion_gui(&estado_aplicar))
            {
                Ok(_) => {
                    mostrar_exito(
                        &estado_aplicar,
                        texto(estado_aplicar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_aplicar));
                }
                Err(error) => mostrar_error(&estado_aplicar, error),
            }
        });

        dialogo.present();
    });

    fila
}

fn opciones_navegador_ids(instaladas: &[String], actual: Option<&str>) -> (Vec<String>, u32) {
    let mut ids = Vec::<String>::new();

    // Una única aplicación instalada puede proponerse, pero nunca se asume.
    if actual.is_none() {
        ids.push(String::new());
    }

    for id in ["firefox", "google-chrome"] {
        let instalada = instaladas.iter().any(|valor| valor == id);
        if instalada || actual == Some(id) {
            ids.push(id.to_string());
        }
    }

    let seleccionado = actual
        .and_then(|valor| ids.iter().position(|id| id == valor))
        .unwrap_or(0) as u32;

    (ids, seleccionado)
}

fn opciones_editor_plasma_ids(actual: Option<&str>) -> (Vec<String>, u32) {
    let mut ids = Vec::<String>::new();

    if actual.is_none() {
        ids.push(String::new());
    }

    ids.push("kwrite".to_string());
    ids.push("kate".to_string());

    let seleccionado = actual
        .and_then(|valor| ids.iter().position(|id| id == valor))
        .unwrap_or(0) as u32;

    (ids, seleccionado)
}

fn argumentos_cambio_roles(
    persona: &str,
    browser_antes: Option<&str>,
    browser_despues: Option<&str>,
    editor_antes: Option<&str>,
    editor_despues: Option<&str>,
    plan: bool,
) -> Option<Vec<String>> {
    let mut argumentos = vec![
        "defaults".to_string(),
        "set".to_string(),
        "--person".to_string(),
        persona.to_string(),
    ];

    if browser_antes != browser_despues {
        if let Some(browser) = browser_despues {
            argumentos.push("--browser".to_string());
            argumentos.push(browser.to_string());
        }
    }

    if editor_antes != editor_despues {
        if let Some(editor) = editor_despues {
            argumentos.push("--plasma-text-editor".to_string());
            argumentos.push(editor.to_string());
        }
    }

    if argumentos.len() == 4 {
        return None;
    }

    argumentos.push(if plan {
        "--plan".to_string()
    } else {
        "--yes".to_string()
    });
    argumentos.push("--json".to_string());

    Some(argumentos)
}

fn agregar_roles_predeterminados(
    pagina: &adw::PreferencesPage,
    estado: Rc<Estado>,
    seleccionados: &[String],
    predeterminados: Option<&Value>,
) {
    let Some(predeterminados) = predeterminados else {
        let grupo = adw::PreferencesGroup::new();
        grupo.set_title(&localizar_visible(
            idioma_actual(),
            "Aplicaciones predeterminadas",
        ));
        grupo.set_description(Some(&localizar_visible(
            idioma_actual(),
            "No pude leer las elecciones predeterminadas del motor. El catálogo de aplicaciones sigue disponible.",
        )));
        pagina.add(&grupo);
        return;
    };

    let personas = predeterminados
        .get("people")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if personas.is_empty() {
        let grupo = adw::PreferencesGroup::new();
        grupo.set_title(&localizar_visible(
            idioma_actual(),
            "Aplicaciones predeterminadas",
        ));
        grupo.set_description(Some(&localizar_visible(
            idioma_actual(),
            "No hay perfiles de persona asignados a este equipo.",
        )));
        pagina.add(&grupo);
        return;
    }

    let varias_personas = personas.len() > 1;

    for persona in personas {
        let Some(persona_id) = persona
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };

        let browser_actual = persona
            .pointer("/requested/browser")
            .and_then(Value::as_str)
            .map(str::to_string);

        let editor_actual = persona
            .pointer("/requested/plasmaTextEditor")
            .and_then(Value::as_str)
            .map(str::to_string);

        let browser_pendiente = persona
            .pointer("/needsChoice/browser")
            .and_then(Value::as_bool)
            .unwrap_or(browser_actual.is_none());

        let editor_pendiente = persona
            .pointer("/needsChoice/plasmaTextEditor")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let mostrar_editor = editor_pendiente || editor_actual.is_some();

        let grupo = adw::PreferencesGroup::new();
        let titulo_roles = if varias_personas {
            format!(
                "{} · {}",
                localizar_visible(idioma_actual(), "Aplicaciones predeterminadas"),
                persona_id
            )
        } else {
            localizar_visible(idioma_actual(), "Aplicaciones predeterminadas")
        };
        grupo.set_title(&titulo_roles);
        grupo.set_description(Some(&localizar_visible(
            idioma_actual(),
            "Elige qué aplicaciones quieres usar normalmente para navegar y editar texto.",
        )));

        let (browser_ids, browser_indice) =
            opciones_navegador_ids(seleccionados, browser_actual.as_deref());

        let browser_labels = browser_ids
            .iter()
            .map(|id| match id.as_str() {
                "" => localizar_visible(idioma_actual(), "Elige una opción"),
                "firefox" => {
                    if seleccionados.iter().any(|valor| valor == "firefox") {
                        "Firefox".to_string()
                    } else {
                        localizar_visible(idioma_actual(), "Firefox — no instalado en este equipo")
                    }
                }
                "google-chrome" => {
                    if seleccionados.iter().any(|valor| valor == "google-chrome") {
                        "Google Chrome".to_string()
                    } else {
                        localizar_visible(
                            idioma_actual(),
                            "Google Chrome — no instalado en este equipo",
                        )
                    }
                }
                otro => otro.to_string(),
            })
            .collect::<Vec<_>>();

        let browser_refs = browser_labels
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();

        let browser_modelo = gtk::StringList::new(&browser_refs);
        let browser = adw::ComboRow::new();
        browser.set_title(&localizar_visible(
            idioma_actual(),
            "Navegador predeterminado",
        ));
        browser.set_model(Some(&browser_modelo));
        browser.set_selected(browser_indice);

        let navegadores_instalados = ["firefox", "google-chrome"]
            .into_iter()
            .filter(|id| seleccionados.iter().any(|valor| valor == id))
            .count();

        let browser_subtitulo = if browser_pendiente && navegadores_instalados == 1 {
            localizar_visible(
                idioma_actual(),
                "Solo hay un navegador instalado. Korunix no decidirá por ti: confirma cuál quieres usar.",
            )
        } else if browser_pendiente {
            localizar_visible(
                idioma_actual(),
                "Elige el navegador que debe abrir enlaces y páginas web.",
            )
        } else {
            localizar_visible(
                idioma_actual(),
                "Abrirá enlaces y páginas web. Las imágenes y los PDF seguirán usando sus propios visores.",
            )
        };

        browser.set_subtitle(&browser_subtitulo);
        grupo.add(&browser);

        let mut editor_ids = Vec::<String>::new();
        let mut editor_combo = None::<adw::ComboRow>;

        if mostrar_editor {
            let (ids, seleccionado) = opciones_editor_plasma_ids(editor_actual.as_deref());

            let labels = ids
                .iter()
                .map(|id| match id.as_str() {
                    "" => localizar_visible(idioma_actual(), "Elige una opción"),
                    "kwrite" => localizar_visible(idioma_actual(), "KWrite — sencillo y ligero"),
                    "kate" => {
                        localizar_visible(idioma_actual(), "Kate — proyectos y trabajo avanzado")
                    }
                    otro => otro.to_string(),
                })
                .collect::<Vec<_>>();

            let refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
            let modelo = gtk::StringList::new(&refs);
            let editor = adw::ComboRow::new();
            editor.set_title(&localizar_visible(
                idioma_actual(),
                "Editor de texto en Plasma",
            ));
            editor.set_subtitle(&localizar_visible(
                idioma_actual(),
                "KWrite es sencillo y ligero para editar archivos de texto. Kate está pensado para muchos archivos, proyectos y herramientas de desarrollo.",
            ));
            editor.set_model(Some(&modelo));
            editor.set_selected(seleccionado);
            grupo.add(&editor);

            editor_ids = ids;
            editor_combo = Some(editor);
        }

        let fila_guardar = adw::ActionRow::new();
        fila_guardar.set_title(&localizar_visible(
            idioma_actual(),
            "Guardar estas elecciones",
        ));
        fila_guardar.set_subtitle(&localizar_visible(
            idioma_actual(),
            "Korunix te mostrará el cambio antes de aplicarlo. No cambiará otras aplicaciones predeterminadas.",
        ));

        let guardar = gtk::Button::with_label(texto(estado.idioma, "save_apply"));
        guardar.add_css_class("suggested-action");
        guardar.set_valign(gtk::Align::Center);
        fila_guardar.add_suffix(&guardar);
        grupo.add(&fila_guardar);

        pagina.add(&grupo);

        let estado_guardar = Rc::clone(&estado);
        let persona_guardar = persona_id.clone();
        let browser_ids_guardar = browser_ids.clone();
        let editor_ids_guardar = editor_ids.clone();
        let browser_antes = browser_actual.clone();
        let editor_antes = editor_actual.clone();

        guardar.connect_clicked(move |boton| {
            let browser_despues = browser_ids_guardar
                .get(browser.selected() as usize)
                .filter(|id| !id.is_empty())
                .cloned();

            if browser_pendiente && browser_despues.is_none() {
                mostrar_error(
                    &estado_guardar,
                    localizar_visible(
                        idioma_actual(),
                        "Elige explícitamente qué navegador debe ser el predeterminado.",
                    ),
                );
                return;
            }

            let editor_despues = editor_combo
                .as_ref()
                .and_then(|fila| editor_ids_guardar.get(fila.selected() as usize))
                .filter(|id| !id.is_empty())
                .cloned();

            if editor_pendiente && editor_despues.is_none() {
                mostrar_error(
                    &estado_guardar,
                    localizar_visible(
                        idioma_actual(),
                        "Elige KWrite o Kate y revisa la diferencia de enfoque antes de continuar.",
                    ),
                );
                return;
            }

            let Some(plan) = argumentos_cambio_roles(
                &persona_guardar,
                browser_antes.as_deref(),
                browser_despues.as_deref(),
                editor_antes.as_deref(),
                editor_despues.as_deref(),
                true,
            ) else {
                mostrar_exito(&estado_guardar, texto(estado_guardar.idioma, "no_change"));
                return;
            };

            if let Err(error) = ejecutar_json_owned(&estado_guardar, &plan) {
                mostrar_error(&estado_guardar, error);
                return;
            }

            let Some(ejecucion) = argumentos_cambio_roles(
                &persona_guardar,
                browser_antes.as_deref(),
                browser_despues.as_deref(),
                editor_antes.as_deref(),
                editor_despues.as_deref(),
                false,
            ) else {
                mostrar_exito(&estado_guardar, texto(estado_guardar.idioma, "no_change"));
                return;
            };

            let cuerpo = format!(
                "{} «{}». {}",
                localizar_visible(
                    idioma_actual(),
                    "Korunix guardará estas aplicaciones predeterminadas para",
                ),
                persona_guardar,
                localizar_visible(
                    idioma_actual(),
                    "Después comprobará la configuración y aplicará únicamente estas decisiones.",
                )
            );

            let dialogo = dialogo_accion(
                boton,
                &estado_guardar,
                &cuerpo,
                texto(estado_guardar.idioma, "save_apply"),
                false,
            );

            let estado_aplicar = Rc::clone(&estado_guardar);
            dialogo.connect_response(None, move |_, respuesta| {
                if respuesta != "apply" {
                    return;
                }

                match ejecutar_json_owned(&estado_aplicar, &ejecucion)
                    .and_then(|_| aplicar_configuracion_gui(&estado_aplicar))
                {
                    Ok(_) => {
                        mostrar_exito(
                            &estado_aplicar,
                            texto(estado_aplicar.idioma, "operation_done"),
                        );
                        recargar(Rc::clone(&estado_aplicar));
                    }
                    Err(error) => mostrar_error(&estado_aplicar, error),
                }
            });

            dialogo.present();
        });
    }
}

fn pagina_aplicaciones(
    estado: Rc<Estado>,
    datos: &Value,
    predeterminados: Option<&Value>,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let seleccionados = ids_aplicaciones_seleccionadas(datos);
    let catalogo = datos
        .get("catalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|valor| valor.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    // El buscador pertenece al inicio de Aplicaciones. Filtra el catálogo
    // curado al escribir y solo consulta catálogos externos cuando la persona
    // lo pide; la fuente concreta nunca se convierte en una pregunta.
    let grupo_busqueda = adw::PreferencesGroup::new();
    grupo_busqueda.set_title(&localizar_visible(idioma_actual(), "Buscar aplicaciones"));
    grupo_busqueda.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Escribe un nombre o una función. Korunix filtra primero sus aplicaciones recomendadas.",
    )));

    let caja_busqueda = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let consulta = gtk::SearchEntry::new();
    consulta.set_placeholder_text(Some(&localizar_visible(
        idioma_actual(),
        "Buscar una aplicación",
    )));
    consulta.set_size_request(-1, 38);

    let buscar_mas = gtk::Button::with_label(&localizar_visible(
        idioma_actual(),
        "Buscar más aplicaciones",
    ));
    buscar_mas.set_sensitive(false);

    caja_busqueda.append(&consulta);
    caja_busqueda.append(&buscar_mas);
    grupo_busqueda.add(&caja_busqueda);
    pagina.add(&grupo_busqueda);

    let grupo_externos = adw::PreferencesGroup::new();
    grupo_externos.set_title(&localizar_visible(
        idioma_actual(),
        "Resultados adicionales",
    ));
    grupo_externos.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Muestra opciones adicionales que Korunix puede instalar de forma compatible.",
    )));
    grupo_externos.set_visible(false);

    let resultados_externos = gtk::Box::new(gtk::Orientation::Vertical, 4);
    grupo_externos.add(&resultados_externos);
    pagina.add(&grupo_externos);

    agregar_roles_predeterminados(&pagina, Rc::clone(&estado), &seleccionados, predeterminados);

    let mut grupos_filtrables =
        Vec::<(adw::PreferencesGroup, Vec<(String, adw::ActionRow)>)>::new();

    for categoria in [
        "Internet y comunicación",
        "Oficina y estudio",
        "Diseño",
        "Multimedia",
        "Juegos",
        "Dispositivos",
        "Desarrollo",
        "Archivos y utilidades",
        "Utilidades",
    ] {
        let grupo_catalogo = adw::PreferencesGroup::new();
        grupo_catalogo.set_title(&localizar_visible(idioma_actual(), categoria));

        let mut filas = Vec::<(String, adw::ActionRow)>::new();

        for id in &catalogo {
            let Some(presentacion) = presentacion_aplicacion(datos, id) else {
                continue;
            };

            if presentacion.categoria != categoria {
                continue;
            }

            let activa = seleccionados.iter().any(|actual| actual == id);
            let huella = format!(
                "{} {} {}",
                presentacion.nombre, presentacion.descripcion, presentacion.categoria
            )
            .to_lowercase();

            let fila = fila_aplicacion(
                Rc::clone(&estado),
                id.clone(),
                "curated".to_string(),
                localizar_visible(idioma_actual(), &presentacion.nombre),
                localizar_visible(idioma_actual(), &presentacion.descripcion),
                activa,
            );

            grupo_catalogo.add(&fila);
            filas.push((huella, fila));
        }

        if !filas.is_empty() {
            pagina.add(&grupo_catalogo);
            grupos_filtrables.push((grupo_catalogo, filas));
        }
    }

    let grupos_filtrables = Rc::new(grupos_filtrables);
    let grupos_busqueda = Rc::clone(&grupos_filtrables);
    let externos_busqueda = grupo_externos.clone();
    let resultados_busqueda = resultados_externos.clone();
    let buscar_mas_busqueda = buscar_mas.clone();

    consulta.connect_search_changed(move |entrada| {
        let consulta = entrada.text().trim().to_lowercase();

        for (grupo, filas) in grupos_busqueda.iter() {
            let mut visibles = 0usize;

            for (huella, fila) in filas {
                let visible = consulta.is_empty() || huella.contains(&consulta);
                fila.set_visible(visible);
                if visible {
                    visibles += 1;
                }
            }

            grupo.set_visible(visibles > 0);
        }

        buscar_mas_busqueda.set_sensitive(!consulta.is_empty());

        while let Some(hijo) = resultados_busqueda.first_child() {
            resultados_busqueda.remove(&hijo);
        }
        externos_busqueda.set_visible(false);
    });

    let estado_buscar = Rc::clone(&estado);
    let consulta_externa = consulta.clone();
    let seleccionados_buscar = seleccionados.clone();

    buscar_mas.connect_clicked(move |_| {
        while let Some(hijo) = resultados_externos.first_child() {
            resultados_externos.remove(&hijo);
        }

        let texto_busqueda = consulta_externa.text().trim().to_string();
        if texto_busqueda.is_empty() {
            grupo_externos.set_visible(false);
            return;
        }

        let mut encontrados = 0usize;
        let mut fuentes_disponibles = 0usize;
        let mut ultimo_error = None::<String>;

        for fuente in ["nixpkgs", "flatpak"] {
            let datos = ejecutar_json(
                &estado_buscar,
                &[
                    "applications",
                    "search",
                    texto_busqueda.as_str(),
                    "--source",
                    fuente,
                    "--json",
                ],
            );

            let datos = match datos {
                Ok(datos) => {
                    fuentes_disponibles += 1;
                    datos
                }
                Err(error) => {
                    ultimo_error = Some(error);
                    continue;
                }
            };

            for resultado in resultados_aplicaciones_externas(&datos, fuente) {
                let token = format!("{}:{}", resultado.fuente, resultado.id);
                let activa = seleccionados_buscar.iter().any(|actual| actual == &token);

                resultados_externos.append(&fila_aplicacion(
                    Rc::clone(&estado_buscar),
                    resultado.id,
                    resultado.fuente,
                    resultado.nombre,
                    resultado.descripcion,
                    activa,
                ));
                encontrados += 1;
            }
        }

        if fuentes_disponibles == 0 {
            if let Some(error) = ultimo_error {
                mostrar_error(&estado_buscar, error);
            }
            grupo_externos.set_visible(false);
            return;
        }

        if encontrados == 0 {
            let vacio = gtk::Label::new(Some(&localizar_visible(
                idioma_actual(),
                "No encontramos más aplicaciones con esa búsqueda.",
            )));
            vacio.set_wrap(true);
            vacio.set_xalign(0.0);
            vacio.add_css_class("dim-label");
            resultados_externos.append(&vacio);
        }

        grupo_externos.set_visible(true);
    });

    pagina
}

fn pagina_escritorio_apariencia(
    estado: Rc<Estado>,
    escritorio: &Value,
    apariencia: &Value,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let estilos = apariencia
        .get("styles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let modos = apariencia
        .get("modes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let estilo_actual = apariencia
        .pointer("/declared/style")
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();

    let modo_actual = apariencia
        .pointer("/declared/mode")
        .and_then(Value::as_str)
        .unwrap_or("auto")
        .to_string();

    let catalogo = escritorio
        .get("catalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let escritorio_actual = escritorio
        .pointer("/desktop/primary")
        .and_then(Value::as_str)
        .unwrap_or("niri")
        .to_string();

    let adicionales_actuales = escritorio
        .pointer("/desktop/additional")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(&localizar_visible(idioma_actual(), "Apariencia"));
    grupo.set_description(Some(
        "La previsualización cambia en esta misma ventana; Guardar y aplicar hace persistente la decisión.",
    ));

    let estilos_humanos = estilos
        .iter()
        .map(|valor| {
            if valor == "everforest" {
                "Everforest".to_string()
            } else {
                "Predeterminada".to_string()
            }
        })
        .collect::<Vec<_>>();
    let estilos_refs = estilos_humanos
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let estilos_modelo = gtk::StringList::new(&estilos_refs);
    let estilo = adw::ComboRow::new();
    estilo.set_title(&localizar_visible(idioma_actual(), "Estilo"));
    estilo.set_model(Some(&estilos_modelo));
    estilo.set_selected(indice(&estilo_actual, &estilos));
    grupo.add(&estilo);

    let modos_humanos = modos
        .iter()
        .map(|valor| match valor.as_str() {
            "light" => "Claro".to_string(),
            "dark" => "Oscuro".to_string(),
            _ => "Automático".to_string(),
        })
        .collect::<Vec<_>>();
    let modos_refs = modos_humanos.iter().map(String::as_str).collect::<Vec<_>>();
    let modos_modelo = gtk::StringList::new(&modos_refs);
    let modo = adw::ComboRow::new();
    modo.set_title(&localizar_visible(idioma_actual(), "Modo"));
    modo.set_model(Some(&modos_modelo));
    modo.set_selected(indice(&modo_actual, &modos));
    grupo.add(&modo);

    let estado_modo = Rc::clone(&estado);
    let modos_preview = modos.clone();
    modo.connect_selected_notify(move |fila| {
        let seleccionado = modos_preview
            .get(fila.selected() as usize)
            .map(String::as_str)
            .unwrap_or("auto");

        let modo = match seleccionado {
            "light" => ModoApariencia::Claro,
            "dark" => ModoApariencia::Oscuro,
            _ => match modo_noctalia_actual() {
                Some(ModoApariencia::Claro) => ModoApariencia::Claro,
                Some(ModoApariencia::Oscuro) => ModoApariencia::Oscuro,
                _ => ModoApariencia::Automatico,
            },
        };

        aplicar_modo(modo);

        if seleccionado == "auto" {
            sincronizar_apariencia_viva(
                estado_modo._apariencia._ajustes_sistema.as_ref(),
                &estado_modo._apariencia._css_noctalia,
            );
        }
    });

    pagina.add(&grupo);

    let grupo_escritorio = adw::PreferencesGroup::new();
    grupo_escritorio.set_title(&localizar_visible(idioma_actual(), "Escritorios"));
    grupo_escritorio.set_description(Some(
        "Elige un escritorio principal y, si quieres, deja otros disponibles para iniciar sesión con ellos.",
    ));

    let catalogo_humano = catalogo
        .iter()
        .map(|id| nombre_escritorio_humano(id))
        .collect::<Vec<_>>();
    let catalogo_refs = catalogo_humano
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let modelo_escritorios = gtk::StringList::new(&catalogo_refs);
    let principal = adw::ComboRow::new();
    principal.set_title(&localizar_visible(idioma_actual(), "Escritorio principal"));
    principal.set_model(Some(&modelo_escritorios));
    principal.set_selected(indice(&escritorio_actual, &catalogo));
    grupo_escritorio.add(&principal);

    let checks = Rc::new(
        catalogo
            .iter()
            .map(|id| {
                let row = adw::ActionRow::new();
                row.set_title(&nombre_escritorio_humano(id));

                let check = gtk::CheckButton::new();
                let es_principal = id == &escritorio_actual;
                check.set_active(
                    es_principal || adicionales_actuales.iter().any(|actual| actual == id),
                );
                check.set_sensitive(!es_principal);
                check.set_valign(gtk::Align::Center);

                if es_principal {
                    row.set_subtitle(&localizar_visible(idioma_actual(), "Principal"));
                }

                row.add_suffix(&check);
                grupo_escritorio.add(&row);
                (id.clone(), check, row)
            })
            .collect::<Vec<_>>(),
    );

    let principal_actual = Rc::new(RefCell::new(escritorio_actual.clone()));
    let checks_principal = Rc::clone(&checks);
    let catalogo_principal = catalogo.clone();
    let principal_actual_cambio = Rc::clone(&principal_actual);

    principal.connect_selected_notify(move |selector| {
        let Some(nuevo) = catalogo_principal.get(selector.selected() as usize) else {
            return;
        };

        let anterior = principal_actual_cambio.borrow().clone();

        for (id, check, row) in checks_principal.iter() {
            if id == nuevo {
                check.set_active(true);
                check.set_sensitive(false);
                row.set_subtitle(&localizar_visible(idioma_actual(), "Principal"));
            } else {
                if id == &anterior {
                    check.set_active(false);
                }
                check.set_sensitive(true);
                row.set_subtitle("");
            }
        }

        *principal_actual_cambio.borrow_mut() = nuevo.clone();
    });

    pagina.add(&grupo_escritorio);

    let grupo_guardar = adw::PreferencesGroup::new();
    let fila_guardar = adw::ActionRow::new();
    fila_guardar.set_title(&localizar_visible(idioma_actual(), "Guardar decisiones"));
    fila_guardar.set_subtitle(&localizar_visible(
        idioma_actual(),
        "Korunix valida primero y crea una sola generación para el conjunto.",
    ));
    let guardar = gtk::Button::with_label(texto(estado.idioma, "save_apply"));
    guardar.add_css_class("suggested-action");
    guardar.set_valign(gtk::Align::Center);
    fila_guardar.add_suffix(&guardar);
    grupo_guardar.add(&fila_guardar);
    pagina.add(&grupo_guardar);

    let estado_guardar = Rc::clone(&estado);
    let estilos_guardar = estilos.clone();
    let modos_guardar = modos.clone();
    let catalogo_guardar = catalogo.clone();

    guardar.connect_clicked(move |boton| {
        let style = estilos_guardar
            .get(estilo.selected() as usize)
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        let mode = modos_guardar
            .get(modo.selected() as usize)
            .cloned()
            .unwrap_or_else(|| "auto".to_string());
        let desktop = catalogo_guardar
            .get(principal.selected() as usize)
            .cloned()
            .unwrap_or_else(|| escritorio_actual.clone());

        let additional = checks
            .iter()
            .filter(|(id, check, _)| check.is_active() && id != &desktop)
            .map(|(id, _, _)| id.clone())
            .collect::<Vec<_>>()
            .join(",");

        let planes = [
            vec![
                "appearance".to_string(),
                "set".to_string(),
                "--style".to_string(),
                style.clone(),
                "--mode".to_string(),
                mode.clone(),
                "--plan".to_string(),
                "--json".to_string(),
            ],
            vec![
                "desktop".to_string(),
                "set-primary".to_string(),
                desktop.clone(),
                "--plan".to_string(),
                "--json".to_string(),
            ],
            vec![
                "desktop".to_string(),
                "set-additional".to_string(),
                additional.clone(),
                "--plan".to_string(),
                "--json".to_string(),
            ],
        ];

        for plan in &planes {
            if let Err(error) = ejecutar_json_owned(&estado_guardar, plan) {
                mostrar_error(&estado_guardar, error);
                return;
            }
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_guardar,
            "¿Guardar y aplicar la apariencia y los escritorios seleccionados?",
            texto(estado_guardar.idioma, "save_apply"),
            false,
        );

        let estado_aplicar = Rc::clone(&estado_guardar);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let acciones = [
                vec![
                    "appearance".to_string(),
                    "set".to_string(),
                    "--style".to_string(),
                    style.clone(),
                    "--mode".to_string(),
                    mode.clone(),
                    "--yes".to_string(),
                    "--json".to_string(),
                ],
                vec![
                    "desktop".to_string(),
                    "set-primary".to_string(),
                    desktop.clone(),
                    "--yes".to_string(),
                    "--json".to_string(),
                ],
                vec![
                    "desktop".to_string(),
                    "set-additional".to_string(),
                    additional.clone(),
                    "--yes".to_string(),
                    "--json".to_string(),
                ],
            ];

            let resultado = (|| -> Result<Value, String> {
                for accion in &acciones {
                    ejecutar_json_owned(&estado_aplicar, accion)?;
                }
                aplicar_configuracion_gui(&estado_aplicar)
            })();

            match resultado {
                Ok(_) => {
                    mostrar_exito(
                        &estado_aplicar,
                        texto(estado_aplicar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_aplicar));
                }
                Err(error) => mostrar_error(&estado_aplicar, error),
            }
        });

        dialogo.present();
    });

    pagina
}

fn pagina_copias_historial(estado: Rc<Estado>, historial: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let grupo_estado = adw::PreferencesGroup::new();
    grupo_estado.set_title(&localizar_visible(idioma_actual(), "Copia portable"));

    let estado_copia = adw::ActionRow::new();
    estado_copia.set_title(&localizar_visible(idioma_actual(), "Última copia portable"));

    match ultima_copia_portable(historial) {
        Some(timestamp) => estado_copia.set_subtitle(&localizar_visible(
            idioma_actual(),
            &format!(
                "{} · No incluye contraseñas ni credenciales.",
                tiempo_relativo(timestamp)
            ),
        )),
        None => estado_copia.set_subtitle(&localizar_visible(
            idioma_actual(),
            "Todavía no has creado una copia portable de esta configuración.",
        )),
    }

    let exportar = gtk::Button::with_label(texto(estado.idioma, "export_backup"));
    exportar.add_css_class("suggested-action");
    exportar.set_valign(gtk::Align::Center);
    estado_copia.add_suffix(&exportar);
    grupo_estado.add(&estado_copia);
    pagina.add(&grupo_estado);

    let estado_exportar = Rc::clone(&estado);
    exportar.connect_clicked(move |boton| {
        if let Err(error) =
            ejecutar_json(&estado_exportar, &["backup", "export", "--plan", "--json"])
        {
            mostrar_error(&estado_exportar, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_exportar,
            "¿Crear ahora una copia portable de la configuración? No incluirá contraseñas ni credenciales.",
            texto(estado_exportar.idioma, "export_backup"),
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_exportar);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            match ejecutar_json(
                &estado_ejecutar,
                &["backup", "export", "--yes", "--json"],
            ) {
                Ok(resultado) => {
                    let salida = resultado
                        .get("output")
                        .and_then(Value::as_str)
                        .unwrap_or("Descargas");
                    mostrar_exito(
                        &estado_ejecutar,
                        &format!("Copia creada en {salida}"),
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });

        dialogo.present();
    });

    let grupo_restaurar = adw::PreferencesGroup::new();
    grupo_restaurar.set_title(&localizar_visible(idioma_actual(), "Restaurar una copia"));
    grupo_restaurar.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Korunix valida la copia y respalda la configuración actual antes de sustituirla.",
    )));

    let ruta = adw::EntryRow::new();
    ruta.set_title(&localizar_visible(idioma_actual(), "Archivo de copia"));
    grupo_restaurar.add(&ruta);

    let restaurar_fila = adw::ActionRow::new();
    restaurar_fila.set_title(&localizar_visible(
        idioma_actual(),
        "Restaurar configuración",
    ));
    let restaurar = gtk::Button::with_label(texto(estado.idioma, "restore_backup"));
    restaurar.add_css_class("destructive-action");
    restaurar.set_valign(gtk::Align::Center);
    restaurar_fila.add_suffix(&restaurar);
    grupo_restaurar.add(&restaurar_fila);
    pagina.add(&grupo_restaurar);

    let estado_restaurar = Rc::clone(&estado);
    restaurar.connect_clicked(move |boton| {
        let archivo = ruta.text().trim().to_string();

        if archivo.is_empty() {
            mostrar_error(
                &estado_restaurar,
                "Indica el archivo de copia que quieres restaurar.",
            );
            return;
        }

        if let Err(error) = ejecutar_json(
            &estado_restaurar,
            &["backup", "restore", archivo.as_str(), "--plan", "--json"],
        ) {
            mostrar_error(&estado_restaurar, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_restaurar,
            "¿Restaurar esta configuración? Korunix guardará primero una copia de seguridad de la configuración actual.",
            texto(estado_restaurar.idioma, "restore_backup"),
            true,
        );

        let estado_ejecutar = Rc::clone(&estado_restaurar);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            match ejecutar_json(
                &estado_ejecutar,
                &["backup", "restore", archivo.as_str(), "--yes", "--json"],
            ) {
                Ok(_) => {
                    mostrar_exito(
                        &estado_ejecutar,
                        "Configuración restaurada y validada. Puedes aplicarla cuando estés listo.",
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });

        dialogo.present();
    });

    let grupo_historial = adw::PreferencesGroup::new();
    grupo_historial.set_title(&localizar_visible(idioma_actual(), "Actividad reciente"));
    grupo_historial.set_description(Some(&localizar_visible(
        idioma_actual(),
        "Cambios realizados o preparados desde Korunix. Los secretos no forman parte de esta lista.",
    )));

    let entradas = historial
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if entradas.is_empty() {
        grupo_historial.add(&fila(
            "Sin actividad todavía",
            "Las acciones que hagas con Korunix aparecerán aquí.",
        ));
    } else {
        for entrada in entradas.iter().rev().take(12) {
            let resumen = entrada
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or("Acción de Korunix");

            let cuando = entrada
                .get("timestamp")
                .and_then(Value::as_u64)
                .map(tiempo_relativo)
                .unwrap_or_default();

            grupo_historial.add(&fila(resumen, cuando));
        }

        if entradas.len() > 12 {
            grupo_historial.set_description(Some(&localizar_visible(
                idioma_actual(),
                &format!(
                    "Mostrando las 12 acciones más recientes de {} registradas.",
                    entradas.len()
                ),
            )));
        }
    }

    pagina.add(&grupo_historial);
    pagina
}

fn pagina_actualizaciones(
    estado: Rc<Estado>,
    channel: &Value,
    plan_actualizacion: &Value,
    aplicaciones: &[String],
    noctalia_relevante: bool,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();

    let grupo_todo = adw::PreferencesGroup::new();
    grupo_todo.set_title(texto(estado.idioma, "update_all"));
    grupo_todo.set_description(Some(
        "Recomendado para la mayoría: Korunix actualiza el sistema y las aplicaciones compatibles como un conjunto coherente.",
    ));

    let fila_todo = adw::ActionRow::new();
    fila_todo.set_title(texto(estado.idioma, "update_all"));
    fila_todo.set_subtitle(&localizar_visible(
        idioma_actual(),
        "Incluye automáticamente las piezas internas que cada actualización necesita.",
    ));
    let todo = gtk::Button::with_label(texto(estado.idioma, "update_all"));
    todo.add_css_class("suggested-action");
    todo.set_valign(gtk::Align::Center);
    fila_todo.add_suffix(&todo);
    grupo_todo.add(&fila_todo);
    pagina.add(&grupo_todo);

    let estado_todo = Rc::clone(&estado);
    todo.connect_clicked(move |boton| {
        if let Err(error) = ejecutar_json(&estado_todo, &["update", "--plan", "--json"]) {
            mostrar_error(&estado_todo, error);
            return;
        }

        let cuerpo = localizar_visible(
            idioma_actual(),
            "¿Buscar y preparar ahora las actualizaciones compatibles? Korunix conservará el estado actual si la nueva combinación no supera la validación.",
        );
        let dialogo = dialogo_accion(
            boton,
            &estado_todo,
            &cuerpo,
            texto(estado_todo.idioma, "update_all"),
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_todo);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            match ejecutar_json(&estado_ejecutar, &["update", "--json"]) {
                Ok(_) => {
                    mostrar_exito(
                        &estado_ejecutar,
                        texto(estado_ejecutar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });
        dialogo.present();
    });

    let grupo_personalizar = adw::PreferencesGroup::new();
    grupo_personalizar.set_title(texto(estado.idioma, "customize_updates"));
    grupo_personalizar.set_description(Some(
        "Elige áreas reconocibles. Korunix mantendrá juntas las dependencias que no pueden actualizarse de forma independiente.",
    ));

    let objetivos = plan_actualizacion
        .get("targets")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|valor| valor.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let unidades = unidades_actualizacion_humanas(&objetivos, aplicaciones, noctalia_relevante);
    let mut checks = Vec::<(Vec<String>, gtk::CheckButton)>::new();

    for unidad in unidades {
        let row = adw::ActionRow::new();
        row.set_title(&localizar_visible(idioma_actual(), &unidad.titulo));
        row.set_subtitle(&localizar_visible(idioma_actual(), &unidad.detalle));

        let check = gtk::CheckButton::new();
        check.set_active(true);
        check.set_valign(gtk::Align::Center);
        row.add_suffix(&check);
        grupo_personalizar.add(&row);
        checks.push((unidad.targets, check));
    }

    let fila_personalizada = adw::ActionRow::new();
    fila_personalizada.set_title(&localizar_visible(idioma_actual(), "Actualizar selección"));
    let personalizada =
        gtk::Button::with_label(&localizar_visible(idioma_actual(), "Actualizar selección"));
    personalizada.set_valign(gtk::Align::Center);
    fila_personalizada.add_suffix(&personalizada);
    grupo_personalizar.add(&fila_personalizada);
    pagina.add(&grupo_personalizar);

    let estado_personalizar = Rc::clone(&estado);
    personalizada.connect_clicked(move |boton| {
        let elegidos = targets_unidades_seleccionadas(&checks);
        let areas = checks
            .iter()
            .filter(|(_, check)| check.is_active())
            .count();

        if elegidos.is_empty() {
            mostrar_error(
                &estado_personalizar,
                "Selecciona al menos un área para actualizar.",
            );
            return;
        }

        let mut plan = vec!["update".to_string()];
        plan.extend(elegidos.clone());
        plan.extend(["--plan".to_string(), "--json".to_string()]);

        if let Err(error) = ejecutar_json_owned(&estado_personalizar, &plan) {
            mostrar_error(&estado_personalizar, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_personalizar,
            &format!(
                "¿Actualizar las {areas} áreas seleccionadas? Korunix incluirá sus dependencias compatibles automáticamente."
            ),
            "Actualizar selección",
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_personalizar);
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            let mut args = vec!["update".to_string()];
            args.extend(elegidos.clone());
            args.push("--json".to_string());

            match ejecutar_json_owned(&estado_ejecutar, &args) {
                Ok(_) => {
                    mostrar_exito(
                        &estado_ejecutar,
                        texto(estado_ejecutar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });
        dialogo.present();
    });

    let grupo_avanzado = adw::PreferencesGroup::new();
    grupo_avanzado.set_title(texto(estado.idioma, "advanced"));
    grupo_avanzado.set_description(Some(
        "Estable prioriza versiones más probadas. Inestable ofrece versiones más recientes y cambia con mayor frecuencia.",
    ));

    let actual = channel
        .get("declared")
        .and_then(Value::as_str)
        .filter(|valor| matches!(*valor, "stable" | "unstable"))
        .unwrap_or("stable")
        .to_string();

    let opciones = gtk::StringList::new(&[
        texto(estado.idioma, "stable"),
        texto(estado.idioma, "unstable"),
    ]);
    let selector = adw::ComboRow::new();
    selector.set_title(texto(estado.idioma, "channel"));
    selector.set_model(Some(&opciones));
    selector.set_selected(if actual == "unstable" { 1 } else { 0 });
    grupo_avanzado.add(&selector);

    let fila_canal = adw::ActionRow::new();
    fila_canal.set_title(&localizar_visible(idioma_actual(), "Cambiar canal"));
    let cambiar = gtk::Button::with_label(texto(estado.idioma, "prepare"));
    cambiar.set_valign(gtk::Align::Center);
    fila_canal.add_suffix(&cambiar);
    grupo_avanzado.add(&fila_canal);
    pagina.add(&grupo_avanzado);

    let estado_canal = Rc::clone(&estado);
    cambiar.connect_clicked(move |boton| {
        let destino = if selector.selected() == 0 {
            "stable"
        } else {
            "unstable"
        };

        if destino == actual {
            mostrar_exito(&estado_canal, texto(estado_canal.idioma, "no_change"));
            return;
        }

        if let Err(error) = ejecutar_json(&estado_canal, &["channel", destino, "--plan", "--json"])
        {
            mostrar_error(&estado_canal, error);
            return;
        }

        let dialogo = dialogo_accion(
            boton,
            &estado_canal,
            &cuerpo_confirmacion_canal(estado_canal.idioma, destino),
            texto(estado_canal.idioma, "apply_change"),
            false,
        );

        let estado_ejecutar = Rc::clone(&estado_canal);
        let destino = destino.to_string();
        dialogo.connect_response(None, move |_, respuesta| {
            if respuesta != "apply" {
                return;
            }

            match ejecutar_motor(&estado_ejecutar, &["channel", destino.as_str(), "--yes"]) {
                Ok(_) => {
                    mostrar_exito(
                        &estado_ejecutar,
                        texto(estado_ejecutar.idioma, "operation_done"),
                    );
                    recargar(Rc::clone(&estado_ejecutar));
                }
                Err(error) => mostrar_error(&estado_ejecutar, error),
            }
        });
        dialogo.present();
    });

    pagina
}

fn recargar(estado: Rc<Estado>) {
    if estado.cargando.get() || estado.ocupado.get() {
        return;
    }

    estado.cargando.set(true);
    mostrar_progreso(&estado, 0, "reading");

    let hardware = consultar(&estado, "hardware");
    mostrar_progreso(&estado, 7, "reading");
    let localization = consultar(&estado, "localization");
    mostrar_progreso(&estado, 10, "reading");
    let people = consultar(&estado, "users");
    mostrar_progreso(&estado, 16, "reading");
    let applications = ejecutar_json(&estado, &["applications", "--json"]);
    let aplicaciones_actualizacion = applications
        .as_ref()
        .ok()
        .map(ids_aplicaciones_seleccionadas)
        .unwrap_or_default();
    let defaults = ejecutar_json(&estado, &["defaults", "--json"]);
    mostrar_progreso(&estado, 22, "reading");
    let desktop = ejecutar_json(&estado, &["desktop", "--json"]);
    let noctalia_actualizacion = desktop
        .as_ref()
        .ok()
        .map(escritorio_usa_noctalia)
        .unwrap_or(false);
    mostrar_progreso(&estado, 28, "reading");
    let appearance = ejecutar_json(&estado, &["appearance", "--json"]);
    mostrar_progreso(&estado, 34, "reading");
    let history = ejecutar_json(&estado, &["history", "--json"]);
    mostrar_progreso(&estado, 40, "reading");
    let channel = consultar(&estado, "channel");
    mostrar_progreso(&estado, 46, "reading");
    let update_plan = ejecutar_json(&estado, &["update", "--plan", "--json"]);
    mostrar_progreso(&estado, 52, "reading");
    let recovery = ejecutar_json(&estado, &["rollback", "--list", "--json"]);
    mostrar_progreso(&estado, 58, "reading");
    let clean = ejecutar_json(&estado, &["clean-preview", "--json"]);
    mostrar_progreso(&estado, 76, "reading");
    let clean_all = ejecutar_json(&estado, &["clean-all-preview", "--json"]);
    mostrar_progreso(&estado, 70, "reading");
    let storage = ejecutar_json(&estado, &["storage", "--list", "--json"]);
    mostrar_progreso(&estado, 64, "reading");
    let firmware_devices = ejecutar_json(&estado, &["firmware", "devices", "--json"]);
    mostrar_progreso(&estado, 82, "reading");
    let firmware_updates = ejecutar_json(&estado, &["firmware", "updates", "--json"]);
    mostrar_progreso(&estado, 88, "reading");
    let media = ejecutar_json(&estado, &["media", "status", "--json"]);
    mostrar_progreso(&estado, 93, "reading");
    let privileges = ejecutar_json(&estado, &["privileges", "--json"]);
    mostrar_progreso(&estado, 97, "reading");

    if let (Ok(hardware), Ok(people), Ok(channel)) = (&hardware, &people, &channel) {
        reemplazar_pagina(
            &estado.stack,
            "summary",
            texto(estado.idioma, "summary"),
            &pagina_resumen(
                Rc::clone(&estado),
                hardware,
                people,
                channel,
                history.as_ref().ok(),
                firmware_updates.as_ref().ok(),
                privileges.as_ref().ok(),
            ),
        );
    } else {
        reemplazar_pagina(
            &estado.stack,
            "summary",
            texto(estado.idioma, "summary"),
            &pagina_error(estado.idioma, texto(estado.idioma, "error")),
        );
    }

    match hardware {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "hardware",
            texto(estado.idioma, "hardware"),
            &pagina_hardware(&estado, &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "hardware",
            texto(estado.idioma, "hardware"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match localization {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "localization",
            texto(estado.idioma, "localization"),
            &pagina_localizacion(Rc::clone(&estado), &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "localization",
            texto(estado.idioma, "localization"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match people {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "people",
            texto(estado.idioma, "people"),
            &pagina_personas(Rc::clone(&estado), &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "people",
            texto(estado.idioma, "people"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match applications {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "applications",
            texto(estado.idioma, "applications"),
            &pagina_aplicaciones(Rc::clone(&estado), &datos, defaults.as_ref().ok()),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "applications",
            texto(estado.idioma, "applications"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match (desktop, appearance) {
        (Ok(escritorio), Ok(apariencia)) => reemplazar_pagina(
            &estado.stack,
            "appearance",
            texto(estado.idioma, "appearance_desktops"),
            &pagina_escritorio_apariencia(Rc::clone(&estado), &escritorio, &apariencia),
        ),
        (Err(error), _) | (_, Err(error)) => reemplazar_pagina(
            &estado.stack,
            "appearance",
            texto(estado.idioma, "appearance_desktops"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match history {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "backups",
            texto(estado.idioma, "backups_history"),
            &pagina_copias_historial(Rc::clone(&estado), &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "backups",
            texto(estado.idioma, "backups_history"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match (channel, update_plan) {
        (Ok(canal), Ok(plan)) => reemplazar_pagina(
            &estado.stack,
            "updates",
            texto(estado.idioma, "updates"),
            &pagina_actualizaciones(
                Rc::clone(&estado),
                &canal,
                &plan,
                &aplicaciones_actualizacion,
                noctalia_actualizacion,
            ),
        ),
        (Err(error), _) | (_, Err(error)) => reemplazar_pagina(
            &estado.stack,
            "updates",
            texto(estado.idioma, "updates"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match media {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "media",
            texto(estado.idioma, "media"),
            &pagina_multimedia(Rc::clone(&estado), &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "media",
            texto(estado.idioma, "media"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match storage {
        Ok(datos) => reemplazar_pagina(
            &estado.stack,
            "storage",
            texto(estado.idioma, "storage"),
            &pagina_almacenamiento(Rc::clone(&estado), &datos),
        ),
        Err(error) => reemplazar_pagina(
            &estado.stack,
            "storage",
            texto(estado.idioma, "storage"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match (firmware_devices, firmware_updates) {
        (Ok(dispositivos), Ok(actualizaciones)) => reemplazar_pagina(
            &estado.stack,
            "firmware",
            texto(estado.idioma, "firmware_updates"),
            &pagina_firmware(Rc::clone(&estado), &dispositivos, &actualizaciones),
        ),
        (Err(error), _) | (_, Err(error)) => reemplazar_pagina(
            &estado.stack,
            "firmware",
            texto(estado.idioma, "firmware_updates"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    match (recovery, clean, clean_all, privileges) {
        (Ok(recuperacion), Ok(limpieza), Ok(limpieza_total), Ok(privilegios)) => {
            reemplazar_pagina(
                &estado.stack,
                "maintenance",
                texto(estado.idioma, "maintenance"),
                &pagina_mantenimiento(
                    Rc::clone(&estado),
                    &recuperacion,
                    &limpieza,
                    &limpieza_total,
                    &privilegios,
                ),
            );
        }
        (Err(error), _, _, _)
        | (_, Err(error), _, _)
        | (_, _, Err(error), _)
        | (_, _, _, Err(error)) => reemplazar_pagina(
            &estado.stack,
            "maintenance",
            texto(estado.idioma, "maintenance"),
            &pagina_error(estado.idioma, &error),
        ),
    }

    mostrar_progreso(&estado, 100, "done");
    ocultar_progreso(&estado);
    estado.cargando.set(false);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModoApariencia {
    Claro,
    Oscuro,
    Automatico,
}

struct AparienciaViva {
    _ajustes_sistema: Option<gio::Settings>,
    _monitores: Vec<gio::FileMonitor>,
    _css_noctalia: gtk::CssProvider,
    _temporizador_noctalia: glib::SourceId,
}

fn directorio_xdg(variable: &str, relativo: &str) -> PathBuf {
    if let Some(valor) = env::var_os(variable) {
        return PathBuf::from(valor);
    }

    let inicio = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    inicio.join(relativo)
}

fn rutas_estado_noctalia() -> Vec<PathBuf> {
    vec![
        directorio_xdg("XDG_STATE_HOME", ".local/state")
            .join("noctalia")
            .join("settings.toml"),
        directorio_xdg("XDG_CONFIG_HOME", ".config")
            .join("noctalia")
            .join("config.toml"),
    ]
}

fn ruta_css_noctalia() -> PathBuf {
    directorio_xdg("XDG_CONFIG_HOME", ".config")
        .join("gtk-4.0")
        .join("noctalia.css")
}

fn valor_toml_simple(linea: &str) -> Option<&str> {
    let (_, valor) = linea.split_once('=')?;
    Some(valor.trim().trim_matches('"').trim_matches('\''))
}

fn modo_noctalia_archivo(ruta: &Path) -> Option<ModoApariencia> {
    let contenido = std::fs::read_to_string(ruta).ok()?;
    let mut seccion = String::new();
    let mut preferencia = None;
    let mut efectivo = None;

    for original in contenido.lines() {
        let linea = original.split('#').next().unwrap_or("").trim();

        if linea.is_empty() {
            continue;
        }

        if linea.starts_with('[') && linea.ends_with(']') {
            seccion = linea
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_ascii_lowercase();
            continue;
        }

        let clave = linea
            .split_once('=')
            .map(|(clave, _)| clave.trim().to_ascii_lowercase())
            .unwrap_or_default();

        if (seccion == "theme" && clave == "mode") || clave == "theme.mode" {
            preferencia = match valor_toml_simple(linea)
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "light" | "claro" => Some(ModoApariencia::Claro),
                "dark" | "oscuro" => Some(ModoApariencia::Oscuro),
                "auto" | "automatic" | "automatico" | "automático" => {
                    Some(ModoApariencia::Automatico)
                }
                _ => preferencia,
            };
            continue;
        }

        let seccion_color = matches!(
            seccion.as_str(),
            "colorschemes" | "color_schemes" | "color-schemes"
        );

        if (seccion_color && matches!(clave.as_str(), "darkmode" | "dark_mode"))
            || matches!(
                clave.as_str(),
                "colorschemes.darkmode" | "color_schemes.dark_mode" | "color-schemes.dark-mode"
            )
        {
            efectivo = match valor_toml_simple(linea)
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "true" | "1" | "yes" | "on" => Some(ModoApariencia::Oscuro),
                "false" | "0" | "no" | "off" => Some(ModoApariencia::Claro),
                _ => efectivo,
            };
            continue;
        }

        if linea.contains("\"darkMode\"") && linea.contains(':') {
            let valor = linea
                .split_once(':')
                .map(|(_, valor)| valor.trim().trim_end_matches(','))
                .unwrap_or("");

            efectivo = match valor {
                "true" => Some(ModoApariencia::Oscuro),
                "false" => Some(ModoApariencia::Claro),
                _ => efectivo,
            };
        }
    }

    efectivo.or(preferencia)
}

fn modo_noctalia_actual() -> Option<ModoApariencia> {
    if let Ok(salida) = Command::new("noctalia")
        .args(["msg", "theme-mode-get"])
        .output()
    {
        if salida.status.success() {
            let valor = String::from_utf8_lossy(&salida.stdout)
                .trim()
                .to_ascii_lowercase();

            match valor.as_str() {
                "light" => return Some(ModoApariencia::Claro),
                "dark" => return Some(ModoApariencia::Oscuro),
                _ => {}
            }
        }
    }

    let mut automatico = false;

    for ruta in rutas_estado_noctalia() {
        match modo_noctalia_archivo(&ruta) {
            Some(ModoApariencia::Claro) => return Some(ModoApariencia::Claro),
            Some(ModoApariencia::Oscuro) => return Some(ModoApariencia::Oscuro),
            Some(ModoApariencia::Automatico) => automatico = true,
            None => {}
        }
    }

    automatico.then_some(ModoApariencia::Automatico)
}

fn gestor_apariencia() -> adw::StyleManager {
    gtk::gdk::Display::default()
        .map(|display| adw::StyleManager::for_display(&display))
        .unwrap_or_else(adw::StyleManager::default)
}

fn modo_sistema(ajustes: Option<&gio::Settings>) -> ModoApariencia {
    let Some(ajustes) = ajustes else {
        return ModoApariencia::Automatico;
    };

    let preferencia: String = ajustes.get("color-scheme");

    match preferencia.as_str() {
        "prefer-dark" => ModoApariencia::Oscuro,
        "prefer-light" => ModoApariencia::Claro,
        _ => ModoApariencia::Automatico,
    }
}

fn aplicar_modo(modo: ModoApariencia) {
    let gestor = gestor_apariencia();

    gestor.set_color_scheme(match modo {
        ModoApariencia::Claro => adw::ColorScheme::ForceLight,
        ModoApariencia::Oscuro => adw::ColorScheme::ForceDark,
        ModoApariencia::Automatico => adw::ColorScheme::Default,
    });
}

fn recargar_css_noctalia(proveedor: &gtk::CssProvider) {
    let ruta = ruta_css_noctalia();

    if ruta.is_file() {
        proveedor.load_from_path(ruta);
    }
}

fn sincronizar_apariencia_viva(
    ajustes_sistema: Option<&gio::Settings>,
    proveedor: &gtk::CssProvider,
) {
    // GTK carga gtk.css al iniciar el proceso; Noctalia reescribe noctalia.css
    // cuando cambia de variante. Recargar este proveedor permite que la misma
    // instancia de Korunix reciba los colores nuevos sin reiniciarse.
    recargar_css_noctalia(proveedor);

    let modo = match modo_noctalia_actual() {
        Some(ModoApariencia::Claro) => ModoApariencia::Claro,
        Some(ModoApariencia::Oscuro) => ModoApariencia::Oscuro,
        Some(ModoApariencia::Automatico) | None => modo_sistema(ajustes_sistema),
    };

    aplicar_modo(modo);
}

fn observar_apariencia_viva() -> AparienciaViva {
    let ajustes_sistema = gio::SettingsSchemaSource::default()
        .and_then(|origen| origen.lookup("org.gnome.desktop.interface", true))
        .filter(|esquema| esquema.has_key("color-scheme"))
        .map(|_| gio::Settings::new("org.gnome.desktop.interface"));

    let proveedor = gtk::CssProvider::new();

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &proveedor,
            gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
        );
    }

    sincronizar_apariencia_viva(ajustes_sistema.as_ref(), &proveedor);

    let pendiente = Rc::new(RefCell::new(None::<glib::SourceId>));

    let programar = {
        let ajustes = ajustes_sistema.clone();
        let proveedor = proveedor.clone();
        let pendiente = Rc::clone(&pendiente);

        move || {
            if let Some(anterior) = pendiente.borrow_mut().take() {
                anterior.remove();
            }

            let ajustes_final = ajustes.clone();
            let proveedor_final = proveedor.clone();
            let pendiente_final = Rc::clone(&pendiente);

            let id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(160), move || {
                    sincronizar_apariencia_viva(ajustes_final.as_ref(), &proveedor_final);
                    pendiente_final.borrow_mut().take();
                });

            *pendiente.borrow_mut() = Some(id);
        }
    };

    if let Some(ajustes) = ajustes_sistema.as_ref() {
        let programar_sistema = programar.clone();
        ajustes.connect_changed(Some("color-scheme"), move |_, _| {
            programar_sistema();
        });
    }

    let mut monitores = Vec::new();
    let mut directorios = Vec::<PathBuf>::new();

    for ruta in rutas_estado_noctalia()
        .into_iter()
        .chain(std::iter::once(ruta_css_noctalia()))
    {
        let Some(directorio) = ruta.parent().map(Path::to_path_buf) else {
            continue;
        };

        if directorios.iter().any(|actual| actual == &directorio) {
            continue;
        }

        directorios.push(directorio.clone());

        let archivo = gio::File::for_path(&directorio);
        let Ok(monitor) =
            archivo.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
        else {
            continue;
        };

        let programar_archivo = programar.clone();
        monitor.connect_changed(move |_, _, _, _| {
            programar_archivo();
        });

        monitores.push(monitor);
    }

    let ajustes_periodicos = ajustes_sistema.clone();
    let proveedor_periodico = proveedor.clone();
    let temporizador_noctalia =
        glib::timeout_add_local(std::time::Duration::from_secs(30), move || {
            sincronizar_apariencia_viva(ajustes_periodicos.as_ref(), &proveedor_periodico);
            glib::ControlFlow::Continue
        });

    AparienciaViva {
        _ajustes_sistema: ajustes_sistema,
        _monitores: monitores,
        _css_noctalia: proveedor,
        _temporizador_noctalia: temporizador_noctalia,
    }
}

fn construir_ventana(app: &adw::Application, raiz: PathBuf, motor: PathBuf) {
    let idioma = idioma_actual();

    let ventana = adw::ApplicationWindow::builder()
        .application(app)
        .title("Korunix")
        .default_width(980)
        .default_height(680)
        .build();

    ventana.set_size_request(360, 520);

    let apariencia = observar_apariencia_viva();

    let stack = gtk::Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    let paginas = [
        ("summary", "summary", "view-grid-symbolic"),
        ("updates", "updates", "software-update-available-symbolic"),
        ("hardware", "hardware", "computer-symbolic"),
        ("media", "media", "audio-speakers-symbolic"),
        ("storage", "storage", "drive-harddisk-symbolic"),
        (
            "firmware",
            "firmware_updates",
            "preferences-system-symbolic",
        ),
        (
            "applications",
            "applications",
            "system-software-install-symbolic",
        ),
        (
            "appearance",
            "appearance_desktops",
            "preferences-desktop-appearance-symbolic",
        ),
        (
            "localization",
            "localization",
            "preferences-desktop-locale-symbolic",
        ),
        ("people", "people", "system-users-symbolic"),
        ("backups", "backups_history", "document-save-symbolic"),
        ("maintenance", "maintenance", "applications-system-symbolic"),
    ];

    for (nombre, clave, _) in paginas {
        let pagina = pagina_error(idioma, texto(idioma, "loading"));
        stack.add_titled(&pagina, Some(nombre), texto(idioma, clave));
    }

    let split = adw::OverlaySplitView::new();
    split.set_min_sidebar_width(220.0);
    split.set_max_sidebar_width(300.0);
    split.set_sidebar_width_fraction(0.28);
    split.set_enable_show_gesture(true);
    split.set_enable_hide_gesture(true);
    split.set_show_sidebar(true);

    let barra_lateral = adw::ToolbarView::new();
    let cabecera_lateral = adw::HeaderBar::new();
    let titulo = adw::WindowTitle::new("Korunix", texto(idioma, "subtitle"));
    cabecera_lateral.set_title_widget(Some(&titulo));
    barra_lateral.add_top_bar(&cabecera_lateral);

    let busqueda_global = gtk::SearchEntry::new();
    busqueda_global.set_placeholder_text(Some(texto(idioma, "global_search")));
    busqueda_global.set_tooltip_text(Some(&localizar_visible(
        idioma_actual(),
        "Buscar ajustes y áreas",
    )));
    busqueda_global.set_can_focus(true);
    busqueda_global.set_hexpand(true);
    busqueda_global.set_size_request(-1, 38);
    busqueda_global.set_margin_top(12);
    busqueda_global.set_margin_start(12);
    busqueda_global.set_margin_end(12);

    let lista = gtk::ListBox::new();
    lista.add_css_class("navigation-sidebar");
    lista.set_selection_mode(gtk::SelectionMode::Single);
    lista.set_margin_top(8);
    lista.set_margin_bottom(12);
    lista.set_margin_start(12);
    lista.set_margin_end(12);

    let mut filas_busqueda = Vec::<(gtk::ListBoxRow, String)>::new();

    for (nombre, clave, icono) in paginas {
        let fila = fila_navegacion(texto(idioma, clave), icono);
        let terminos = format!(
            "{} {}",
            texto(idioma, clave),
            terminos_busqueda_pagina(nombre)
        )
        .to_lowercase();

        lista.append(&fila);
        filas_busqueda.push((fila, terminos));
    }

    let busqueda_vacia = gtk::Label::new(Some(&localizar_visible(
        idioma_actual(),
        "No encontramos un área con ese nombre.",
    )));
    busqueda_vacia.set_wrap(true);
    busqueda_vacia.set_justify(gtk::Justification::Center);
    busqueda_vacia.add_css_class("dim-label");
    busqueda_vacia.set_margin_top(28);
    busqueda_vacia.set_margin_bottom(28);
    busqueda_vacia.set_margin_start(18);
    busqueda_vacia.set_margin_end(18);
    busqueda_vacia.set_visible(false);

    let desplazamiento_lateral = gtk::ScrolledWindow::new();
    desplazamiento_lateral.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    desplazamiento_lateral.set_vexpand(true);
    desplazamiento_lateral.set_child(Some(&lista));

    let contenido_lateral = gtk::Box::new(gtk::Orientation::Vertical, 0);
    contenido_lateral.append(&busqueda_global);
    contenido_lateral.append(&busqueda_vacia);
    contenido_lateral.append(&desplazamiento_lateral);
    barra_lateral.set_content(Some(&contenido_lateral));

    let pagina_lateral = adw::NavigationPage::new(&barra_lateral, "Korunix");

    let barra_contenido = adw::ToolbarView::new();
    let cabecera_contenido = adw::HeaderBar::new();
    cabecera_contenido.set_show_back_button(false);

    let menu_secciones = gtk::Button::from_icon_name("view-list-symbolic");
    menu_secciones.set_tooltip_text(Some(texto(idioma, "sections")));
    menu_secciones.set_visible(false);

    let split_menu = split.clone();
    menu_secciones.connect_clicked(move |_| {
        split_menu.set_show_sidebar(true);
    });

    cabecera_contenido.pack_start(&menu_secciones);

    let refresh = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh.set_tooltip_text(Some(texto(idioma, "refresh")));
    cabecera_contenido.pack_end(&refresh);
    barra_contenido.add_top_bar(&cabecera_contenido);

    let progreso = gtk::Revealer::new();
    progreso.set_transition_type(gtk::RevealerTransitionType::SlideDown);

    let progreso_caja = gtk::Box::new(gtk::Orientation::Vertical, 4);
    progreso_caja.set_margin_top(8);
    progreso_caja.set_margin_bottom(8);
    progreso_caja.set_margin_start(18);
    progreso_caja.set_margin_end(18);

    let progreso_texto = gtk::Label::new(None);
    progreso_texto.set_xalign(0.0);

    let progreso_barra = gtk::ProgressBar::new();
    progreso_barra.set_show_text(true);
    progreso_barra.set_text(Some("0%"));

    progreso_caja.append(&progreso_texto);
    progreso_caja.append(&progreso_barra);
    progreso.set_child(Some(&progreso_caja));

    let toast = adw::ToastOverlay::new();
    toast.set_vexpand(true);
    toast.set_child(Some(&stack));

    let contenido = gtk::Box::new(gtk::Orientation::Vertical, 0);
    contenido.append(&progreso);
    contenido.append(&toast);
    barra_contenido.set_content(Some(&contenido));

    let pagina_contenido = adw::NavigationPage::new(&barra_contenido, texto(idioma, "summary"));

    split.set_sidebar(Some(&pagina_lateral));
    split.set_content(Some(&pagina_contenido));

    let stack_clon = stack.clone();
    let split_clon = split.clone();
    let pagina_contenido_clon = pagina_contenido.clone();

    let filas_busqueda = Rc::new(filas_busqueda);

    {
        let lista_busqueda = lista.clone();
        let filas_busqueda = Rc::clone(&filas_busqueda);
        let busqueda_vacia = busqueda_vacia.clone();

        busqueda_global.connect_search_changed(move |entrada| {
            let consulta = entrada.text().trim().to_lowercase();
            let mut primera_visible = None::<gtk::ListBoxRow>;

            for (fila, terminos) in filas_busqueda.iter() {
                let visible = consulta.is_empty() || terminos.contains(&consulta);
                fila.set_visible(visible);

                if visible && primera_visible.is_none() {
                    primera_visible = Some(fila.clone());
                }
            }

            busqueda_vacia.set_visible(primera_visible.is_none());

            let seleccion_visible = lista_busqueda
                .selected_row()
                .map(|fila| fila.is_visible())
                .unwrap_or(false);

            if !seleccion_visible {
                lista_busqueda.select_row(primera_visible.as_ref());
            }
        });
    }

    {
        let split_busqueda = split.clone();
        let lista_busqueda = lista.clone();

        busqueda_global.connect_activate(move |_| {
            if lista_busqueda.selected_row().is_some() && split_busqueda.is_collapsed() {
                split_busqueda.set_show_sidebar(false);
            }
        });
    }

    lista.connect_row_selected(move |_, fila| {
        let Some(fila) = fila else {
            return;
        };

        let indice = fila.index() as usize;
        let Some((nombre, clave, _)) = paginas.get(indice) else {
            return;
        };

        stack_clon.set_visible_child_name(nombre);
        pagina_contenido_clon.set_title(texto(idioma, clave));

        if split_clon.is_collapsed() {
            split_clon.set_show_sidebar(false);
        }
    });

    if let Some(fila) = lista.row_at_index(0) {
        lista.select_row(Some(&fila));
    }

    let condicion = adw::BreakpointCondition::parse("max-width: 819px")
        .expect("La condición adaptable de Korunix debe ser válida.");
    let breakpoint = adw::Breakpoint::new(condicion);

    let split_estrecho = split.clone();
    let menu_estrecho = menu_secciones.clone();
    breakpoint.connect_apply(move |_| {
        split_estrecho.set_collapsed(true);
        split_estrecho.set_show_sidebar(false);
        menu_estrecho.set_visible(true);
    });

    let split_ancho = split.clone();
    let menu_ancho = menu_secciones.clone();
    breakpoint.connect_unapply(move |_| {
        split_ancho.set_collapsed(false);
        split_ancho.set_show_sidebar(true);
        menu_ancho.set_visible(false);
    });

    ventana.add_breakpoint(breakpoint);
    ventana.set_content(Some(&split));

    let estado = Rc::new(Estado {
        raiz,
        motor,
        idioma,
        stack,
        navegacion: lista.clone(),
        pagina_contenido: pagina_contenido.clone(),
        toast,
        progreso,
        progreso_barra,
        progreso_texto,
        cargando: Cell::new(false),
        ocupado: Cell::new(false),
        camara_preview_activa: Cell::new(false),
        _apariencia: apariencia,
    });

    let estado_clon = Rc::clone(&estado);
    refresh.connect_clicked(move |_| {
        recargar(Rc::clone(&estado_clon));
    });

    recargar(estado);
    ventana.present();
}

#[cfg(test)]
mod pruebas_roles_predeterminados_gui {
    use super::*;

    #[test]
    fn historial_encuentra_la_ultima_copia_portable() {
        let historial = serde_json::json!({
            "entries": [
                {"timestamp": 10, "kind": "backup-export"},
                {"timestamp": 20, "kind": "applications-prepared"},
                {"timestamp": 30, "kind": "backup-export"}
            ]
        });

        assert_eq!(ultima_copia_portable(&historial), Some(30));
    }

    #[test]
    fn tiempo_relativo_es_humano() {
        assert_eq!(tiempo_relativo_desde(1_000, 1_030), "Ahora");
        assert_eq!(tiempo_relativo_desde(1_000, 1_120), "Hace 2 min");
        assert_eq!(tiempo_relativo_desde(1_000, 8_200), "Hace 2 h");
        assert_eq!(
            tiempo_relativo_desde(1_000, 1_000 + 2 * 86_400),
            "Hace 2 días"
        );
    }

    #[test]
    fn resumen_solo_alerta_con_evidencia() {
        let ahora = 4_000_000;
        let historial_reciente = serde_json::json!({
            "entries": [{
                "timestamp": ahora - 10 * 86_400,
                "kind": "backup-export"
            }]
        });
        let firmware = serde_json::json!({"devices": []});
        let privilegios = serde_json::json!({"guiUsable": true});

        assert!(asuntos_resumen(
            Some(&historial_reciente),
            Some(&firmware),
            Some(&privilegios),
            ahora
        )
        .is_empty());

        let firmware_pendiente = serde_json::json!({"devices": [{"id": "uno"}]});
        let historial_vacio = serde_json::json!({"entries": []});
        let privilegios_inutiles = serde_json::json!({"guiUsable": false});
        let asuntos = asuntos_resumen(
            Some(&historial_vacio),
            Some(&firmware_pendiente),
            Some(&privilegios_inutiles),
            ahora,
        );

        assert_eq!(asuntos.len(), 3);
        assert_eq!(asuntos[0].destino, "backups");
        assert_eq!(asuntos[1].destino, "firmware");
        assert_eq!(asuntos[2].destino, "maintenance");
    }

    #[test]
    fn mapa_de_navegacion_cubre_las_doce_paginas() {
        for nombre in [
            "summary",
            "updates",
            "hardware",
            "media",
            "storage",
            "firmware",
            "applications",
            "appearance",
            "localization",
            "people",
            "backups",
            "maintenance",
        ] {
            assert!(indice_pagina(nombre).is_some(), "{nombre}");
        }
    }

    #[test]
    fn busqueda_global_encuentra_decisiones_humanas() {
        assert!(terminos_busqueda_pagina("applications").contains("firefox"));
        assert!(terminos_busqueda_pagina("applications").contains("navegador"));
        assert!(terminos_busqueda_pagina("localization").contains("teclado"));
        assert!(terminos_busqueda_pagina("storage").contains("usb"));
        assert!(terminos_busqueda_pagina("hardware").contains("memoria"));
    }

    #[test]
    fn presentacion_curada_oculta_dependencias_y_describe_apps() {
        let datos = serde_json::json!({
            "presentation": {
                "firefox": {
                    "name": "Firefox",
                    "description": "Navegador web para páginas y enlaces.",
                    "category": "Internet y comunicación"
                }
            }
        });

        let firefox = presentacion_aplicacion(&datos, "firefox")
            .expect("Firefox debe tener presentación humana");
        assert_eq!(firefox.nombre, "Firefox");
        assert!(!firefox.descripcion.trim().is_empty());
        assert_eq!(firefox.categoria, "Internet y comunicación");

        assert!(presentacion_aplicacion(&datos, "aagl").is_none());
        assert!(presentacion_aplicacion(&datos, "kate").is_none());
        assert!(presentacion_aplicacion(&datos, "android-tools").is_none());
    }

    #[test]
    fn actualizaciones_agrupan_dependencias_sin_perder_targets() {
        let objetivos = vec![
            "aagl".to_string(),
            "aaglStable".to_string(),
            "alejandra".to_string(),
            "hatter".to_string(),
            "millennium".to_string(),
            "nix-flatpak".to_string(),
            "nixpkgs".to_string(),
            "nixpkgsStable".to_string(),
            "noctalia".to_string(),
            "spicetify-nix".to_string(),
        ];
        let aplicaciones = vec![
            "genshin-impact".to_string(),
            "spotify".to_string(),
            "steam".to_string(),
        ];

        let unidades = unidades_actualizacion_humanas(&objetivos, &aplicaciones, true);
        let titulos = unidades
            .iter()
            .map(|unidad| unidad.titulo.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            titulos,
            [
                "Sistema y aplicaciones",
                "Juegos instalados",
                "Spotify",
                "Steam",
                "Noctalia"
            ]
        );

        for titulo in &titulos {
            assert!(![
                "aagl",
                "aaglStable",
                "alejandra",
                "hatter",
                "millennium",
                "nix-flatpak",
                "nixpkgs",
                "nixpkgsStable",
                "spicetify-nix"
            ]
            .contains(titulo));
        }

        let mut reconstruidos = unidades
            .into_iter()
            .flat_map(|unidad| unidad.targets)
            .collect::<Vec<_>>();
        reconstruidos.sort();

        let mut esperados = objetivos;
        esperados.sort();

        assert_eq!(reconstruidos, esperados);
    }

    #[test]
    fn aagl_no_se_convierte_en_fila_si_no_hay_juego_elegido() {
        let objetivos = vec![
            "aagl".to_string(),
            "aaglStable".to_string(),
            "nixpkgs".to_string(),
        ];

        let unidades = unidades_actualizacion_humanas(&objetivos, &[], false);
        assert_eq!(unidades.len(), 1);
        assert_eq!(unidades[0].titulo, "Sistema y aplicaciones");
        assert_eq!(unidades[0].targets.len(), 3);
    }

    #[test]
    fn noctalia_solo_es_unidad_visible_si_hay_niri_o_hyprland() {
        let objetivos = vec!["nixpkgs".to_string(), "noctalia".to_string()];

        let sin_noctalia = unidades_actualizacion_humanas(&objetivos, &[], false);
        assert_eq!(sin_noctalia.len(), 1);
        assert_eq!(sin_noctalia[0].titulo, "Sistema y aplicaciones");
        assert_eq!(sin_noctalia[0].targets.len(), 2);

        let con_noctalia = unidades_actualizacion_humanas(&objetivos, &[], true);
        assert_eq!(
            con_noctalia
                .iter()
                .map(|unidad| unidad.titulo.as_str())
                .collect::<Vec<_>>(),
            ["Sistema y aplicaciones", "Noctalia"]
        );
    }

    #[test]
    fn resultados_externos_conservan_fuente_sin_usarla_como_nombre() {
        let datos = serde_json::json!({
            "results": {
                "legacyPackages.x86_64-linux.demo": {
                    "pname": "Demo",
                    "description": "Aplicación de ejemplo"
                }
            }
        });

        let resultados = resultados_aplicaciones_externas(&datos, "nixpkgs");
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].nombre, "Demo");
        assert_eq!(resultados[0].fuente, "nixpkgs");
        assert_eq!(resultados[0].descripcion, "Aplicación de ejemplo");
    }

    #[test]
    fn un_navegador_instalado_no_se_asume() {
        let instaladas = vec!["firefox".to_string()];
        let (ids, seleccionado) = opciones_navegador_ids(&instaladas, None);

        assert_eq!(seleccionado, 0);
        assert_eq!(ids, vec!["".to_string(), "firefox".to_string()]);
    }

    #[test]
    fn navegador_actual_se_conserva_aunque_no_este_instalado() {
        let instaladas = vec!["firefox".to_string()];
        let (ids, seleccionado) = opciones_navegador_ids(&instaladas, Some("google-chrome"));

        assert_eq!(ids[seleccionado as usize], "google-chrome");
    }

    #[test]
    fn cambio_parcial_no_reenvia_el_editor() {
        let argumentos = argumentos_cambio_roles(
            "persona",
            Some("firefox"),
            Some("google-chrome"),
            Some("kate"),
            Some("kate"),
            true,
        )
        .expect("debe existir un cambio");

        assert!(argumentos
            .windows(2)
            .any(|par| { par == ["--browser".to_string(), "google-chrome".to_string()] }));
        assert!(!argumentos
            .iter()
            .any(|valor| valor == "--plasma-text-editor"));
        assert!(argumentos.iter().any(|valor| valor == "--plan"));
        assert!(argumentos.iter().any(|valor| valor == "--json"));
    }

    #[test]
    fn editor_plasma_no_se_asume_si_falta_eleccion() {
        let (ids, seleccionado) = opciones_editor_plasma_ids(None);

        assert_eq!(seleccionado, 0);
        assert_eq!(ids[0], "");
        assert_eq!(ids[1], "kwrite");
        assert_eq!(ids[2], "kate");
    }
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APPLICATION_ID)
        .flags(gio::ApplicationFlags::empty())
        .build();

    app.connect_activate(|app| {
        let raiz = match raiz_proyecto() {
            Ok(valor) => valor,
            Err(error) => {
                eprintln!("Korunix: {error}",);
                return;
            }
        };

        let motor = match motor(&raiz) {
            Ok(valor) => valor,
            Err(error) => {
                eprintln!("Korunix: {error}",);
                return;
            }
        };

        construir_ventana(app, raiz, motor);
    });

    app.run()
}
