#!/usr/bin/env python3
"""Selector compacto de sesiones Korunix para la pantalla integrada de Veskalia."""
import subprocess
import threading

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, GLib, Gtk


CSS = """
* { font-family: Cantarell, Sans, sans-serif; }
window, .korunix-root { background: #101116; color: #f4f5f8; }
.kicker { color: #8e96a8; font-size: 13px; font-weight: 700; letter-spacing: 2px; }
.heading { color: #ffffff; font-size: 30px; font-weight: 800; }
.subtitle { color: #a3a9b8; font-size: 15px; }
.session-card {
  background: #191c24;
  color: #f6f7fa;
  border: 1px solid #343946;
  border-radius: 18px;
  padding: 20px;
  min-width: 255px;
  min-height: 208px;
  transition: background 120ms ease, border-color 120ms ease;
}
.session-card:hover, .session-card:focus-visible {
  background: #222a38;
  border: 2px solid #67b7ff;
}
.session-card:active { background: #29384b; }
.card-icon { color: #8dcaff; margin-bottom: 10px; }
.card-title { color: #ffffff; font-size: 20px; font-weight: 750; }
.card-description { color: #a8afbd; font-size: 13px; }
.status { color: #aab2c1; font-size: 13px; }
.hint { color: #7f8797; font-size: 12px; }
button { outline-color: #67b7ff; }
"""

SESSIONS = [
    {
        "title": "GNOME",
        "description": "Escritorio completo",
        "icon": "computer-symbolic",
        "session": "gnome.desktop",
        "kind": "desktop",
    },
    {
        "title": "Niri",
        "description": "Niri + Noctalia",
        "icon": "view-grid-symbolic",
        "session": "niri.desktop",
        "kind": "desktop",
    },
    {
        "title": "Gaming Mode",
        "description": "Steam Deck UI",
        "icon": "input-gaming-symbolic",
        "session": "gamescope-wayland.desktop",
        "kind": "gaming",
    },
]


class SessionSelector(Gtk.Application):
    def __init__(self):
        super().__init__(application_id="dev.korunix.sessionselector")
        self.window = None
        self.status = None
        self.buttons = []
        self.switching = False

    def do_activate(self):
        if self.window is not None:
            self.window.present()
            return

        display = Gdk.Display.get_default()
        if display:
            provider = Gtk.CssProvider()
            provider.load_from_data(CSS)
            Gtk.StyleContext.add_provider_for_display(
                display, provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
            )

        self.window = Gtk.ApplicationWindow(application=self)
        self.window.set_title("Korunix — selector de sesiones")
        self.window.set_decorated(False)
        self.window.fullscreen()
        self.window.add_css_class("korunix-root")

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        root.set_margin_top(22)
        root.set_margin_bottom(18)
        root.set_margin_start(28)
        root.set_margin_end(28)
        root.set_valign(Gtk.Align.FILL)
        root.add_css_class("korunix-root")

        brand_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        brand_icon = Gtk.Image.new_from_icon_name("input-gaming-symbolic")
        brand_icon.set_pixel_size(28)
        brand_icon.add_css_class("card-icon")
        brand_row.append(brand_icon)
        brand = Gtk.Label(label="KORUNIX")
        brand.add_css_class("kicker")
        brand.set_halign(Gtk.Align.START)
        brand_row.append(brand)
        root.append(brand_row)

        title = Gtk.Label(label="Elige tu experiencia")
        title.add_css_class("heading")
        title.set_halign(Gtk.Align.START)
        title.set_xalign(0)
        root.append(title)

        subtitle = Gtk.Label(label="Cambia de entorno sin interrumpir tu sesión con un inicio de sesión manual.")
        subtitle.add_css_class("subtitle")
        subtitle.set_halign(Gtk.Align.START)
        subtitle.set_xalign(0)
        subtitle.set_wrap(True)
        root.append(subtitle)

        cards = Gtk.FlowBox()
        cards.set_homogeneous(True)
        cards.set_selection_mode(Gtk.SelectionMode.NONE)
        cards.set_max_children_per_line(3)
        cards.set_min_children_per_line(1)
        cards.set_row_spacing(12)
        cards.set_column_spacing(12)
        cards.set_valign(Gtk.Align.CENTER)
        cards.set_vexpand(True)
        cards.set_halign(Gtk.Align.FILL)
        cards.set_hexpand(True)

        for session in SESSIONS:
            button = Gtk.Button()
            button.add_css_class("session-card")
            button.set_focusable(True)
            button.set_hexpand(True)
            button.set_vexpand(False)
            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=7)
            card.set_halign(Gtk.Align.CENTER)
            card.set_valign(Gtk.Align.CENTER)

            icon = Gtk.Image.new_from_icon_name(session["icon"])
            icon.set_pixel_size(42)
            icon.add_css_class("card-icon")
            icon.set_halign(Gtk.Align.CENTER)
            card.append(icon)

            label = Gtk.Label(label=session["title"])
            label.add_css_class("card-title")
            label.set_halign(Gtk.Align.CENTER)
            card.append(label)

            desc = Gtk.Label(label=session["description"])
            desc.add_css_class("card-description")
            desc.set_halign(Gtk.Align.CENTER)
            desc.set_wrap(True)
            card.append(desc)

            button.set_child(card)
            button.connect("clicked", self.choose_session, session)
            cards.append(button)
            self.buttons.append(button)

        root.append(cards)

        self.status = Gtk.Label(label="Selecciona con el tacto, el ratón o el teclado.")
        self.status.add_css_class("status")
        self.status.set_halign(Gtk.Align.CENTER)
        self.status.set_wrap(True)
        root.append(self.status)

        hint = Gtk.Label(label="ESC: volver a Gaming Mode  ·  Korunix")
        hint.add_css_class("hint")
        hint.set_halign(Gtk.Align.CENTER)
        root.append(hint)

        keys = Gtk.EventControllerKey()
        keys.connect("key-pressed", self.on_key_pressed)
        self.window.add_controller(keys)
        self.window.set_child(root)
        self.window.present()
        GLib.timeout_add(250, self.focus_first)

    def focus_first(self):
        if self.buttons:
            self.buttons[0].grab_focus()
        return GLib.SOURCE_REMOVE

    def on_key_pressed(self, _controller, keyval, _keycode, _state):
        if keyval == Gdk.KEY_Escape and not self.switching:
            self.choose_session(None, next(s for s in SESSIONS if s["kind"] == "gaming"))
            return True
        return False

    def choose_session(self, _button, session):
        if self.switching:
            return
        self.switching = True
        for button in self.buttons:
            button.set_sensitive(False)
        self.status.set_label(f"Abriendo {session['title']}…")

        threading.Thread(
            target=self._request_switch,
            args=(session,),
            daemon=True,
        ).start()

    def _run_steamosctl(self, command):
        return subprocess.run(
            ["steamosctl", *command],
            capture_output=True,
            text=True,
            timeout=8,
            check=False,
        )

    def _request_switch(self, session):
        """Ask SteamOS Manager for a session transition using its real CLI API.

        switch-to-desktop-mode has no session argument. The requested .desktop
        entry must first be selected as the default desktop session; switching
        modes then stages it as the next one-shot SDDM login.
        """
        try:
            if session["kind"] == "gaming":
                result = self._run_steamosctl(["switch-to-game-mode"])
            else:
                result = self._run_steamosctl(
                    ["set-default-desktop-session", session["session"]]
                )
                if result.returncode == 0:
                    result = self._run_steamosctl(["switch-to-desktop-mode"])
            code = result.returncode
            message = (result.stderr or result.stdout or "").strip()
        except Exception as exc:
            code = 127
            message = str(exc)
        GLib.idle_add(self._switch_result, code, message, session["title"])

    def _switch_result(self, code, message, label):
        if code != 0:
            self.switching = False
            for button in self.buttons:
                button.set_sensitive(True)
            detail = message[-220:] if message else f"steamosctl devolvió {code}"
            self.status.set_label(f"No se pudo abrir {label}: {detail}")
            return GLib.SOURCE_REMOVE

        # steamosctl gestiona la transición y el cierre de la sesión a través de
        # SteamOS Manager/SDDM. No terminamos la sesión manualmente: hacerlo en
        # paralelo puede competir con el cambio de sesión y dejar la pantalla negra.
        self.status.set_label("Cambio solicitado. Esperando al gestor de sesiones…")
        return GLib.SOURCE_REMOVE


if __name__ == "__main__":
    app = SessionSelector()
    raise SystemExit(app.run())
