"""Textos propios de Korunix para la primera interfaz gráfica.

El español es la fuente canónica. Inglés y húngaro permiten validar desde el
inicio que la composición y el motor de textos no dependan de una sola lengua.
"""

from __future__ import annotations

import os

from korunix_backend import normalize_language


CATALOGS: dict[str, dict[str, str]] = {
    "es": {
        "app.name": "Korunix",
        "app.subtitle": "Centro de control del sistema",
        "search.placeholder": "Buscar en Korunix",
        "search.accessible": "Buscar ajustes y áreas",
        "search.empty": "No encontramos un área con ese nombre.",
        "search.terms.summary": "estado configuración coherencia equipo",
        "search.terms.localization": (
            "idioma país región fecha hora teclado método de entrada"
        ),
        "search.terms.hardware": (
            "equipo hardware procesador memoria gráficos red controlador"
        ),
        "search.terms.people": "persona usuario cuenta perfil administrador",
        "nav.summary": "Resumen",
        "nav.summary.note": "Estado general del equipo",
        "nav.localization": "Idioma y región",
        "nav.localization.note": "Idioma, país, hora y teclado",
        "nav.hardware": "Este equipo",
        "nav.hardware.note": "Hardware detectado",
        "nav.people": "Personas",
        "nav.people.note": "Cuentas y perfiles",
        "action.refresh": "Actualizar estado",
        "nav.updates": "Actualizaciones",
        "nav.updates.note": "Canal y ritmo de actualizaciones",
        "search.terms.updates": (
            "actualizaciones canal estable inestable nixos paquetes software"
        ),
        "channels.title": "Actualizaciones",
        "channels.description": (
            "Elige el ritmo de cambios que prefieres para este equipo."
        ),
        "channels.current.group": "Canal actual",
        "channels.current": "Canal",
        "channels.nixos": "Versión de NixOS",
        "channels.nixpkgs": "Paquetes base",
        "channels.aagl": "AAGL",
        "channels.choice.group": "Cambiar de canal",
        "channels.choice": "Canal",
        "channels.choice.note": (
            "Seleccionar una opción no modifica el sistema."
        ),
        "channels.stable": "Estable",
        "channels.unstable": "Inestable",
        "channels.prepare": "Preparar cambio",
        "channels.preparing": "Preparando…",
        "channels.nochange": "Este canal ya está seleccionado",
        "channels.action.note": "Valida el cambio sin aplicarlo.",
        "channels.selected": "Sin cambios pendientes",
        "channels.selected.note": "Este equipo ya usa el canal seleccionado.",
        "channels.error": "No pudimos preparar el canal",
        "loading.title": "Leyendo este equipo",
        "loading.body": "Korunix reúne el estado local sin modificar nada.",
        "error.title": "No pudimos leer el estado",
        "error.body": "Comprueba que este equipo tenga una configuración de Korunix disponible.",
        "error.partial.title": "Parte del estado no está disponible",
        "error.partial.body": "Puedes seguir consultando las áreas que sí respondieron.",
        "error.area.localization": "No pudimos leer idioma y región",
        "error.area.hardware": "No pudimos leer este equipo",
        "error.area.people": "No pudimos leer las personas",
        "error.area.body": "Nada cambió. Actualiza el estado para volver a intentarlo.",
        "summary.title": "Resumen",
        "summary.description": "Lo esencial de este equipo en un solo lugar.",
        "summary.current": "Configuración actual",
        "summary.mode": "Modo",
        "summary.mode.value": "Consulta de solo lectura",
        "summary.desktop": "Escritorio activo",
        "summary.language": "Idioma del sistema",
        "summary.machine": "Tipo de equipo",
        "summary.people": "Personas detectadas",
        "summary.state": "Coherencia",
        "summary.state.ready": "La información disponible es coherente",
        "summary.state.warning": "Hay datos que conviene revisar",
        "summary.state.incomplete": "Falta información de una o más áreas",
        "summary.hero.ready": "Este equipo está listo",
        "summary.hero.ready.body": (
            "Korunix pudo leer la configuración disponible y no encontró "
            "contradicciones."
        ),
        "summary.hero.warning": "Hay una decisión que conviene revisar",
        "summary.hero.warning.body": (
            "El equipo sigue seguro. Revisa la información señalada antes de "
            "aplicar cambios."
        ),
        "summary.hero.incomplete": "Parte del estado no está disponible",
        "summary.hero.incomplete.body": (
            "Puedes seguir consultando las áreas que respondieron; nada fue "
            "modificado."
        ),
        "localization.title": "Idioma y región",
        "localization.description": (
            "Estas decisiones permanecen separadas y pueden combinarse libremente."
        ),
        "localization.language.group": "Idioma",
        "localization.interface": "Interfaz de Korunix",
        "localization.system": "Idioma del sistema",
        "localization.region.group": "Región y formatos",
        "localization.country": "País o región",
        "localization.formats": "Fechas, números y unidades",
        "localization.timezone": "Hora local",
        "localization.input.group": "Entrada",
        "localization.keyboards": "Teclados",
        "localization.input_method": "Método de entrada",
        "localization.supported": "Idiomas disponibles en Noctalia",
        "hardware.title": "Este equipo",
        "hardware.description": "Información detectada localmente, sin consultar Internet.",
        "hardware.machine.group": "Equipo",
        "hardware.type": "Tipo",
        "hardware.vendor": "Fabricante",
        "hardware.model": "Modelo",
        "hardware.platform.group": "Plataforma",
        "hardware.architecture": "Arquitectura",
        "hardware.firmware": "Arranque",
        "hardware.processor": "Procesador",
        "hardware.processors": "Procesadores lógicos",
        "hardware.memory": "Memoria",
        "hardware.graphics.group": "Gráficos y red",
        "hardware.graphics": "Gráficos",
        "hardware.drivers": "Controladores activos",
        "hardware.network": "Red",
        "people.title": "Personas",
        "people.description": "Cuentas reales del equipo y su relación con Korunix.",
        "people.summary.group": "Resumen",
        "people.total": "Personas detectadas",
        "people.adopted": "Preparadas por Korunix",
        "people.administrators": "Administradores",
        "people.accounts.group": "Cuentas de este equipo",
        "people.role.admin": "Puede administrar este equipo",
        "people.role.standard": "Cuenta estándar",
        "people.status.adopted": "Preparada por Korunix",
        "people.status.adoptable": "Disponible para preparar",
        "people.status.profile-available": "Tiene un perfil disponible",
        "value.unavailable": "No disponible",
        "value.none": "Ninguno",
        "value.yes": "Sí",
        "value.no": "No",
    },
    "en": {
        "app.name": "Korunix",
        "app.subtitle": "System control center",
        "search.placeholder": "Search Korunix",
        "search.accessible": "Search settings and areas",
        "search.empty": "We could not find an area with that name.",
        "search.terms.summary": "status configuration consistency computer",
        "search.terms.localization": (
            "language country region date time keyboard input method"
        ),
        "search.terms.hardware": (
            "computer hardware processor memory graphics network driver"
        ),
        "search.terms.people": "person user account profile administrator",
        "nav.summary": "Overview",
        "nav.summary.note": "General computer status",
        "nav.localization": "Language and region",
        "nav.localization.note": "Language, country, time and keyboard",
        "nav.hardware": "This computer",
        "nav.hardware.note": "Detected hardware",
        "nav.people": "People",
        "nav.people.note": "Accounts and profiles",
        "action.refresh": "Refresh status",
        "nav.updates": "Updates",
        "nav.updates.note": "Update channel and release pace",
        "search.terms.updates": (
            "updates channel stable unstable nixos packages software"
        ),
        "channels.title": "Updates",
        "channels.description": (
            "Choose the pace of changes you prefer for this computer."
        ),
        "channels.current.group": "Current channel",
        "channels.current": "Channel",
        "channels.nixos": "NixOS version",
        "channels.nixpkgs": "Base packages",
        "channels.aagl": "AAGL",
        "channels.choice.group": "Change channel",
        "channels.choice": "Channel",
        "channels.choice.note": (
            "Selecting an option does not modify the system."
        ),
        "channels.stable": "Stable",
        "channels.unstable": "Unstable",
        "channels.prepare": "Prepare change",
        "channels.preparing": "Preparing…",
        "channels.nochange": "This channel is already selected",
        "channels.action.note": "Validates the change without applying it.",
        "channels.selected": "No pending changes",
        "channels.selected.note": "This computer already uses the selected channel.",
        "channels.error": "We could not prepare the channel",
        "loading.title": "Reading this computer",
        "loading.body": "Korunix gathers local status without changing anything.",
        "error.title": "We could not read the status",
        "error.body": "Make sure this computer has a Korunix configuration available.",
        "error.partial.title": "Some status is unavailable",
        "error.partial.body": "You can keep viewing the areas that did respond.",
        "error.area.localization": "We could not read language and region",
        "error.area.hardware": "We could not read this computer",
        "error.area.people": "We could not read people",
        "error.area.body": "Nothing changed. Refresh the status to try again.",
        "summary.title": "Overview",
        "summary.description": "The essentials of this computer in one place.",
        "summary.current": "Current configuration",
        "summary.mode": "Mode",
        "summary.mode.value": "Read-only inspection",
        "summary.desktop": "Active desktop",
        "summary.language": "System language",
        "summary.machine": "Computer type",
        "summary.people": "People detected",
        "summary.state": "Consistency",
        "summary.state.ready": "Available information is consistent",
        "summary.state.warning": "Some information should be reviewed",
        "summary.state.incomplete": "Information from one or more areas is missing",
        "summary.hero.ready": "This computer is ready",
        "summary.hero.ready.body": (
            "Korunix read the available configuration and found no contradictions."
        ),
        "summary.hero.warning": "A choice should be reviewed",
        "summary.hero.warning.body": (
            "The computer remains safe. Review the highlighted information "
            "before applying changes."
        ),
        "summary.hero.incomplete": "Some status is unavailable",
        "summary.hero.incomplete.body": (
            "You can keep viewing the areas that responded; nothing was changed."
        ),
        "localization.title": "Language and region",
        "localization.description": "These choices remain separate and can be combined freely.",
        "localization.language.group": "Language",
        "localization.interface": "Korunix interface",
        "localization.system": "System language",
        "localization.region.group": "Region and formats",
        "localization.country": "Country or region",
        "localization.formats": "Dates, numbers and units",
        "localization.timezone": "Local time",
        "localization.input.group": "Input",
        "localization.keyboards": "Keyboards",
        "localization.input_method": "Input method",
        "localization.supported": "Languages available in Noctalia",
        "hardware.title": "This computer",
        "hardware.description": "Information detected locally, without using the Internet.",
        "hardware.machine.group": "Computer",
        "hardware.type": "Type",
        "hardware.vendor": "Manufacturer",
        "hardware.model": "Model",
        "hardware.platform.group": "Platform",
        "hardware.architecture": "Architecture",
        "hardware.firmware": "Boot",
        "hardware.processor": "Processor",
        "hardware.processors": "Logical processors",
        "hardware.memory": "Memory",
        "hardware.graphics.group": "Graphics and network",
        "hardware.graphics": "Graphics",
        "hardware.drivers": "Active drivers",
        "hardware.network": "Network",
        "people.title": "People",
        "people.description": "Real computer accounts and their relationship with Korunix.",
        "people.summary.group": "Overview",
        "people.total": "People detected",
        "people.adopted": "Prepared by Korunix",
        "people.administrators": "Administrators",
        "people.accounts.group": "Accounts on this computer",
        "people.role.admin": "Can administer this computer",
        "people.role.standard": "Standard account",
        "people.status.adopted": "Prepared by Korunix",
        "people.status.adoptable": "Available to prepare",
        "people.status.profile-available": "Has an available profile",
        "value.unavailable": "Unavailable",
        "value.none": "None",
        "value.yes": "Yes",
        "value.no": "No",
    },
    "hu": {
        "app.name": "Korunix",
        "app.subtitle": "Rendszervezérlő központ",
        "search.placeholder": "Keresés a Korunixban",
        "search.accessible": "Beállítások és területek keresése",
        "search.empty": "Nem található ilyen nevű terület.",
        "search.terms.summary": "állapot beállítás konzisztencia számítógép",
        "search.terms.localization": (
            "nyelv ország régió dátum idő billentyűzet beviteli mód"
        ),
        "search.terms.hardware": (
            "számítógép hardver processzor memória grafika hálózat illesztőprogram"
        ),
        "search.terms.people": "személy felhasználó fiók profil rendszergazda",
        "nav.summary": "Áttekintés",
        "nav.summary.note": "A számítógép általános állapota",
        "nav.localization": "Nyelv és régió",
        "nav.localization.note": "Nyelv, ország, idő és billentyűzet",
        "nav.hardware": "Ez a számítógép",
        "nav.hardware.note": "Észlelt hardver",
        "nav.people": "Felhasználók",
        "nav.people.note": "Fiókok és profilok",
        "action.refresh": "Állapot frissítése",
        "nav.updates": "Frissítések",
        "nav.updates.note": "Frissítési csatorna és kiadási ütem",
        "search.terms.updates": (
            "frissítés csatorna stabil instabil nixos csomagok szoftver"
        ),
        "channels.title": "Frissítések",
        "channels.description": (
            "Válaszd ki a számítógéphez kívánt frissítési ütemet."
        ),
        "channels.current.group": "Jelenlegi csatorna",
        "channels.current": "Csatorna",
        "channels.nixos": "NixOS-verzió",
        "channels.nixpkgs": "Alapcsomagok",
        "channels.aagl": "AAGL",
        "channels.choice.group": "Csatornaváltás",
        "channels.choice": "Csatorna",
        "channels.choice.note": (
            "A kiválasztás önmagában nem módosítja a rendszert."
        ),
        "channels.stable": "Stabil",
        "channels.unstable": "Instabil",
        "channels.prepare": "Váltás előkészítése",
        "channels.preparing": "Előkészítés…",
        "channels.nochange": "Ez a csatorna már ki van választva",
        "channels.action.note": "Ellenőrzi a váltást annak alkalmazása nélkül.",
        "channels.selected": "Nincs függőben lévő változás",
        "channels.selected.note": "A számítógép már ezt a csatornát használja.",
        "channels.error": "Nem sikerült előkészíteni a csatornát",
        "loading.title": "A számítógép beolvasása",
        "loading.body": "A Korunix változtatás nélkül összegyűjti a helyi állapotot.",
        "error.title": "Nem sikerült beolvasni az állapotot",
        "error.body": (
            "Ellenőrizd, hogy elérhető-e a Korunix konfigurációja ezen a "
            "számítógépen."
        ),
        "error.partial.title": "Az állapot egy része nem érhető el",
        "error.partial.body": "A válaszoló területek továbbra is megtekinthetők.",
        "error.area.localization": "A nyelv és a régió nem olvasható",
        "error.area.hardware": "A számítógép adatai nem olvashatók",
        "error.area.people": "A felhasználók nem olvashatók",
        "error.area.body": "Semmi sem változott. Próbáld újra az állapot frissítésével.",
        "summary.title": "Áttekintés",
        "summary.description": "A számítógép legfontosabb adatai egy helyen.",
        "summary.current": "Jelenlegi konfiguráció",
        "summary.mode": "Mód",
        "summary.mode.value": "Csak olvasható ellenőrzés",
        "summary.desktop": "Aktív asztali környezet",
        "summary.language": "Rendszernyelv",
        "summary.machine": "Számítógép típusa",
        "summary.people": "Észlelt felhasználók",
        "summary.state": "Konzisztencia",
        "summary.state.ready": "Az elérhető adatok konzisztensek",
        "summary.state.warning": "Néhány adatot érdemes ellenőrizni",
        "summary.state.incomplete": "Egy vagy több terület adatai hiányoznak",
        "summary.hero.ready": "A számítógép készen áll",
        "summary.hero.ready.body": (
            "A Korunix beolvasta az elérhető konfigurációt, és nem talált "
            "ellentmondást."
        ),
        "summary.hero.warning": "Egy beállítást érdemes ellenőrizni",
        "summary.hero.warning.body": (
            "A számítógép továbbra is biztonságos. A módosítások alkalmazása "
            "előtt ellenőrizd a jelzett adatot."
        ),
        "summary.hero.incomplete": "Az állapot egy része nem érhető el",
        "summary.hero.incomplete.body": (
            "A válaszoló területek továbbra is megtekinthetők; semmi sem változott."
        ),
        "localization.title": "Nyelv és régió",
        "localization.description": (
            "Ezek a beállítások különállók és szabadon kombinálhatók."
        ),
        "localization.language.group": "Nyelv",
        "localization.interface": "Korunix felülete",
        "localization.system": "Rendszernyelv",
        "localization.region.group": "Régió és formátumok",
        "localization.country": "Ország vagy régió",
        "localization.formats": "Dátumok, számok és mértékegységek",
        "localization.timezone": "Helyi idő",
        "localization.input.group": "Bevitel",
        "localization.keyboards": "Billentyűzetek",
        "localization.input_method": "Beviteli mód",
        "localization.supported": "A Noctaliában elérhető nyelvek",
        "hardware.title": "Ez a számítógép",
        "hardware.description": "Helyben észlelt adatok, internetkapcsolat nélkül.",
        "hardware.machine.group": "Számítógép",
        "hardware.type": "Típus",
        "hardware.vendor": "Gyártó",
        "hardware.model": "Modell",
        "hardware.platform.group": "Platform",
        "hardware.architecture": "Architektúra",
        "hardware.firmware": "Rendszerindítás",
        "hardware.processor": "Processzor",
        "hardware.processors": "Logikai processzorok",
        "hardware.memory": "Memória",
        "hardware.graphics.group": "Grafika és hálózat",
        "hardware.graphics": "Grafika",
        "hardware.drivers": "Aktív illesztőprogramok",
        "hardware.network": "Hálózat",
        "people.title": "Felhasználók",
        "people.description": "A számítógép valós fiókjai és kapcsolatuk a Korunixszal.",
        "people.summary.group": "Áttekintés",
        "people.total": "Észlelt felhasználók",
        "people.adopted": "Korunix által előkészítve",
        "people.administrators": "Rendszergazdák",
        "people.accounts.group": "A számítógép fiókjai",
        "people.role.admin": "Kezelheti ezt a számítógépet",
        "people.role.standard": "Általános fiók",
        "people.status.adopted": "Korunix által előkészítve",
        "people.status.adoptable": "Előkészíthető",
        "people.status.profile-available": "Elérhető profil tartozik hozzá",
        "value.unavailable": "Nem érhető el",
        "value.none": "Nincs",
        "value.yes": "Igen",
        "value.no": "Nem",
    },
}


def detect_interface_language() -> str:
    """Decide el idioma antes de construir cualquier widget."""

    explicit = os.environ.get("KORUNIX_LANGUAGE")
    language_environment = (
        explicit
        or os.environ.get("LC_ALL")
        or os.environ.get("LC_MESSAGES")
        or os.environ.get("LANGUAGE")
        or os.environ.get("LANG")
    )
    detected = normalize_language(language_environment)
    return detected if detected in CATALOGS else "es"


class Translator:
    def __init__(self, language: str | None = None) -> None:
        requested = normalize_language(language) if language else detect_interface_language()
        self.language = requested if requested in CATALOGS else "es"

    def text(self, key: str, **values: object) -> str:
        source = CATALOGS[self.language].get(key, CATALOGS["es"].get(key, key))
        return source.format(**values)
