#!/usr/bin/env python3
"""Primera interfaz gráfica de Korunix: adaptable, local y de solo lectura."""

from __future__ import annotations

import threading
import sys
from pathlib import Path
from typing import Any

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import Adw, Gio, GLib, Gtk  # noqa: E402

from korunix_backend import (  # noqa: E402
    BackendError,
    Snapshot,
    find_project_root,
    human_language,
    load_snapshot,
    present_hardware,
    present_localization,
    present_users,
)
from korunix_i18n import Translator  # noqa: E402


APPLICATION_ID = "io.github.koruninn.Korunix"


class KorunixWindow(Adw.ApplicationWindow):
    def __init__(self, application: Adw.Application) -> None:
        super().__init__(application=application)

        # El idioma queda resuelto antes de construir el primer widget.
        self.translator = Translator()
        self.text = self.translator.text
        self.project_root: Path | None = None
        self.loading = False
        self.rows: dict[str, Adw.ActionRow] = {}
        self.pages_by_row: dict[Adw.ActionRow, str] = {}

        self.set_title(self.text("app.name"))
        self.set_default_size(1080, 720)
        self.set_size_request(360, 560)

        self._load_styles()
        self._build_shell()
        self._show_loading_pages()

        self._load_state()

    def _load_styles(self) -> None:
        provider = Gtk.CssProvider()
        provider.load_from_path(str(Path(__file__).with_name("style.css")))
        Gtk.StyleContext.add_provider_for_display(
            self.get_display(),
            provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

    def _build_shell(self) -> None:
        self.split_view = Adw.NavigationSplitView()
        self.split_view.set_min_sidebar_width(260)
        self.split_view.set_max_sidebar_width(340)
        self.split_view.set_sidebar_width_fraction(0.30)

        sidebar_toolbar = Adw.ToolbarView()
        sidebar_header = Adw.HeaderBar()
        sidebar_header.set_title_widget(
            Adw.WindowTitle(
                title=self.text("app.name"),
                subtitle=self.text("app.subtitle"),
            )
        )
        sidebar_toolbar.add_top_bar(sidebar_header)

        sidebar_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        sidebar_box.add_css_class("korunix-sidebar")
        self.navigation = Gtk.ListBox()
        self.navigation.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self.navigation.add_css_class("navigation-sidebar")
        self.navigation.set_margin_top(12)
        self.navigation.set_margin_bottom(12)
        self.navigation.set_margin_start(12)
        self.navigation.set_margin_end(12)
        self.navigation.connect("row-selected", self._on_navigation_selected)

        navigation_items = (
            (
                "summary",
                "view-grid-symbolic",
                "nav.summary",
                "nav.summary.note",
            ),
            (
                "localization",
                "preferences-desktop-locale-symbolic",
                "nav.localization",
                "nav.localization.note",
            ),
            (
                "hardware",
                "computer-symbolic",
                "nav.hardware",
                "nav.hardware.note",
            ),
            (
                "people",
                "system-users-symbolic",
                "nav.people",
                "nav.people.note",
            ),
        )

        for name, icon_name, title_key, note_key in navigation_items:
            row = Adw.ActionRow(
                title=self.text(title_key),
                subtitle=self.text(note_key),
            )
            row.set_title_lines(2)
            row.set_subtitle_lines(2)
            row.set_activatable(True)
            row.add_prefix(Gtk.Image.new_from_icon_name(icon_name))
            self.navigation.append(row)
            self.rows[name] = row
            self.pages_by_row[row] = name

        sidebar_box.append(self.navigation)
        sidebar_toolbar.set_content(sidebar_box)
        sidebar_page = Adw.NavigationPage(
            child=sidebar_toolbar,
            title=self.text("app.name"),
        )
        self.split_view.set_sidebar(sidebar_page)

        content_toolbar = Adw.ToolbarView()
        content_header = Adw.HeaderBar()
        self.content_title = Adw.WindowTitle(title=self.text("nav.summary"))
        content_header.set_title_widget(self.content_title)

        self.refresh_button = Gtk.Button.new_from_icon_name("view-refresh-symbolic")
        self.refresh_button.set_tooltip_text(self.text("action.refresh"))
        self.refresh_button.connect("clicked", lambda _button: self._load_state())
        content_header.pack_end(self.refresh_button)
        content_toolbar.add_top_bar(content_header)

        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.stack.set_transition_duration(180)
        content_toolbar.set_content(self.stack)

        content_page = Adw.NavigationPage(
            child=content_toolbar,
            title=self.text("nav.summary"),
        )
        self.split_view.set_content(content_page)
        self.set_content(self.split_view)

        # NavigationSplitView expone el estado colapsado, pero el punto de
        # quiebre es quien debe activarlo. Así la ventana angosta muestra una
        # sola página completa y Adw.HeaderBar ofrece el botón de regreso.
        breakpoint = Adw.Breakpoint.new(
            Adw.BreakpointCondition.parse("max-width: 819px")
        )
        breakpoint.add_setter(self.split_view, "collapsed", True)
        self.add_breakpoint(breakpoint)

        first_row = self.rows["summary"]
        self.navigation.select_row(first_row)

    def _on_navigation_selected(
        self,
        _list_box: Gtk.ListBox,
        row: Adw.ActionRow | None,
    ) -> None:
        if row is None:
            return

        page_name = self.pages_by_row[row]
        if self.stack.get_child_by_name(page_name) is not None:
            self.stack.set_visible_child_name(page_name)

        self.content_title.set_title(row.get_title())
        if self.split_view.get_collapsed():
            self.split_view.set_show_content(True)

    def _loading_page(self) -> Adw.StatusPage:
        page = Adw.StatusPage(
            title=self.text("loading.title"),
            description=self.text("loading.body"),
            icon_name="view-refresh-symbolic",
        )
        spinner = Gtk.Spinner(spinning=True)
        spinner.set_size_request(32, 32)
        page.set_child(spinner)
        return page

    def _show_loading_pages(self) -> None:
        for name in self.rows:
            self._replace_stack_page(name, self._loading_page())

    def _replace_stack_page(self, name: str, widget: Gtk.Widget) -> None:
        previous = self.stack.get_child_by_name(name)
        was_visible = self.stack.get_visible_child_name() == name
        if previous is not None:
            self.stack.remove(previous)

        self.stack.add_named(widget, name)
        if was_visible or self.stack.get_visible_child() is None:
            self.stack.set_visible_child_name(name)

    def _load_state(self) -> None:
        if self.loading:
            return

        self.loading = True
        self.refresh_button.set_sensitive(False)
        self._show_loading_pages()

        worker = threading.Thread(target=self._load_state_worker, daemon=True)
        worker.start()

    def _load_state_worker(self) -> None:
        try:
            root = self.project_root or find_project_root()
            snapshot = load_snapshot(root)
        except BackendError as error:
            GLib.idle_add(self._show_total_error, str(error))
            return

        self.project_root = snapshot.root
        GLib.idle_add(self._show_snapshot, snapshot)

    def _show_total_error(self, _detail: str) -> bool:
        self.loading = False
        self.refresh_button.set_sensitive(True)

        for name in self.rows:
            page = Adw.StatusPage(
                title=self.text("error.title"),
                description=self.text("error.body"),
                icon_name="dialog-error-symbolic",
            )
            retry = Gtk.Button(label=self.text("action.refresh"))
            retry.add_css_class("suggested-action")
            retry.connect("clicked", lambda _button: self._load_state())
            page.set_child(retry)
            self._replace_stack_page(name, page)
        return GLib.SOURCE_REMOVE

    def _show_snapshot(self, snapshot: Snapshot) -> bool:
        self.loading = False
        self.refresh_button.set_sensitive(True)

        for area, detail in snapshot.errors.items():
            print(f"Korunix: {area}: {detail}", file=sys.stderr)

        localization = (
            present_localization(
                snapshot.data["localization"], self.translator.language
            )
            if "localization" in snapshot.data
            else None
        )
        hardware = (
            present_hardware(snapshot.data["hardware"], self.translator.language)
            if "hardware" in snapshot.data
            else None
        )
        people = (
            present_users(snapshot.data["users"])
            if "users" in snapshot.data
            else None
        )

        self._replace_stack_page(
            "summary",
            self._build_summary_page(localization, hardware, people, snapshot),
        )
        self._replace_stack_page(
            "localization",
            self._build_localization_page(
                localization,
                snapshot.errors.get("localization"),
            ),
        )
        self._replace_stack_page(
            "hardware",
            self._build_hardware_page(hardware, snapshot.errors.get("hardware")),
        )
        self._replace_stack_page(
            "people",
            self._build_people_page(people, snapshot.errors.get("users")),
        )
        return GLib.SOURCE_REMOVE

    def _preferences_page(self, title: str, description: str) -> Adw.PreferencesPage:
        page = Adw.PreferencesPage(title=title, description=description)
        page.set_vexpand(True)
        return page

    def _value_row(
        self,
        title: str,
        value: object,
        icon_name: str | None = None,
    ) -> Adw.ActionRow:
        rendered = (
            str(value)
            if value not in (None, "")
            else self.text("value.unavailable")
        )
        row = Adw.ActionRow(title=title, subtitle=rendered)
        row.set_title_lines(0)
        row.set_subtitle_lines(0)
        row.set_subtitle_selectable(True)
        if icon_name:
            row.add_prefix(Gtk.Image.new_from_icon_name(icon_name))
        return row

    def _partial_error_group(self) -> Adw.PreferencesGroup:
        group = Adw.PreferencesGroup(title=self.text("error.partial.title"))
        row = Adw.ActionRow(
            title=self.text("error.partial.body"),
            icon_name="dialog-warning-symbolic",
        )
        group.add(row)
        return group

    def _build_summary_page(
        self,
        localization: dict[str, Any] | None,
        hardware: dict[str, Any] | None,
        people: dict[str, Any] | None,
        snapshot: Snapshot,
    ) -> Adw.PreferencesPage:
        page = self._preferences_page(
            self.text("summary.title"), self.text("summary.description")
        )
        group = Adw.PreferencesGroup(title=self.text("summary.current"))
        group.add(
            self._value_row(
                self.text("summary.mode"),
                self.text("summary.mode.value"),
                "emblem-system-symbolic",
            )
        )

        if localization:
            group.add(
                self._value_row(
                    self.text("summary.desktop"),
                    localization["desktop"],
                    "preferences-desktop-display-symbolic",
                )
            )
            group.add(
                self._value_row(
                    self.text("summary.language"),
                    localization["systemLanguage"],
                    "preferences-desktop-locale-symbolic",
                )
            )

        if hardware:
            group.add(
                self._value_row(
                    self.text("summary.machine"),
                    hardware["type"],
                    "computer-symbolic",
                )
            )

        if people:
            group.add(
                self._value_row(
                    self.text("summary.people"),
                    people["humanAccounts"],
                    "system-users-symbolic",
                )
            )
        page.add(group)

        contradictions = len(localization["contradictions"]) if localization else 0
        consistency = Adw.PreferencesGroup(title=self.text("summary.state"))
        ready = snapshot.complete and contradictions == 0
        if snapshot.errors:
            consistency_key = "summary.state.incomplete"
        elif contradictions:
            consistency_key = "summary.state.warning"
        else:
            consistency_key = "summary.state.ready"
        consistency.add(
            self._value_row(
                self.text("summary.state"),
                self.text(consistency_key),
                "emblem-ok-symbolic" if ready else "dialog-warning-symbolic",
            )
        )
        page.add(consistency)

        if snapshot.errors:
            page.add(self._partial_error_group())
        return page

    def _build_localization_page(
        self, data: dict[str, Any] | None, error: str | None
    ) -> Gtk.Widget:
        if data is None:
            return self._area_error_page("localization", error)

        page = self._preferences_page(
            self.text("localization.title"),
            self.text("localization.description"),
        )
        language_group = Adw.PreferencesGroup(
            title=self.text("localization.language.group")
        )
        language_group.add(
            self._value_row(
                self.text("localization.interface"),
                human_language(self.translator.language, self.translator.language),
                "preferences-desktop-locale-symbolic",
            )
        )
        language_group.add(
            self._value_row(
                self.text("localization.system"), data["systemLanguage"]
            )
        )
        language_group.add(
            self._value_row(
                self.text("localization.supported"),
                len(data["supportedInterfaceLanguages"]),
            )
        )
        page.add(language_group)

        region_group = Adw.PreferencesGroup(
            title=self.text("localization.region.group")
        )
        region_group.add(
            self._value_row(self.text("localization.country"), data["region"])
        )
        region_group.add(
            self._value_row(self.text("localization.formats"), data["formats"])
        )
        region_group.add(
            self._value_row(self.text("localization.timezone"), data["timeZone"])
        )
        page.add(region_group)

        input_group = Adw.PreferencesGroup(
            title=self.text("localization.input.group")
        )
        input_group.add(
            self._value_row(
                self.text("localization.keyboards"), " · ".join(data["keyboards"])
            )
        )
        input_group.add(
            self._value_row(
                self.text("localization.input_method"), data["inputMethod"]
            )
        )
        page.add(input_group)

        if error:
            page.add(self._partial_error_group())
        return page

    def _build_hardware_page(
        self, data: dict[str, Any] | None, error: str | None
    ) -> Gtk.Widget:
        if data is None:
            return self._area_error_page("hardware", error)

        page = self._preferences_page(
            self.text("hardware.title"), self.text("hardware.description")
        )
        machine_group = Adw.PreferencesGroup(
            title=self.text("hardware.machine.group")
        )
        machine_group.add(self._value_row(self.text("hardware.type"), data["type"]))
        machine_group.add(
            self._value_row(self.text("hardware.vendor"), data["vendor"])
        )
        machine_group.add(
            self._value_row(self.text("hardware.model"), data["model"])
        )
        page.add(machine_group)

        platform_group = Adw.PreferencesGroup(
            title=self.text("hardware.platform.group")
        )
        platform_group.add(
            self._value_row(
                self.text("hardware.architecture"), data["architecture"]
            )
        )
        platform_group.add(
            self._value_row(self.text("hardware.firmware"), data["firmware"])
        )
        platform_group.add(
            self._value_row(self.text("hardware.processor"), data["processor"])
        )
        platform_group.add(
            self._value_row(
                self.text("hardware.processors"), data["logicalProcessors"]
            )
        )
        platform_group.add(
            self._value_row(self.text("hardware.memory"), data["memory"])
        )
        page.add(platform_group)

        graphics_group = Adw.PreferencesGroup(
            title=self.text("hardware.graphics.group")
        )
        graphics_group.add(
            self._value_row(
                self.text("hardware.graphics"), " · ".join(data["graphics"])
            )
        )
        graphics_group.add(
            self._value_row(
                self.text("hardware.drivers"),
                " · ".join(data["graphicsDrivers"])
                or self.text("value.none"),
            )
        )
        graphics_group.add(
            self._value_row(
                self.text("hardware.network"),
                " · ".join(data["network"]) or self.text("value.unavailable"),
            )
        )
        page.add(graphics_group)

        if error:
            page.add(self._partial_error_group())
        return page

    def _build_people_page(
        self, data: dict[str, Any] | None, error: str | None
    ) -> Gtk.Widget:
        if data is None:
            return self._area_error_page("people", error)

        page = self._preferences_page(
            self.text("people.title"), self.text("people.description")
        )
        summary = Adw.PreferencesGroup(title=self.text("people.summary.group"))
        summary.add(
            self._value_row(self.text("people.total"), data["humanAccounts"])
        )
        summary.add(
            self._value_row(self.text("people.adopted"), data["adoptedAccounts"])
        )
        summary.add(
            self._value_row(
                self.text("people.administrators"),
                data["detectedAdministrators"],
            )
        )
        page.add(summary)

        accounts = Adw.PreferencesGroup(
            title=self.text("people.accounts.group")
        )
        for account in data["accounts"]:
            role = self.text(
                "people.role.admin"
                if account["administrator"]
                else "people.role.standard"
            )
            status_key = f"people.status.{account['status']}"
            row = Adw.ActionRow(
                title=account["name"],
                subtitle=f"{role} · {self.text(status_key)}",
            )
            row.set_title_lines(0)
            row.set_subtitle_lines(0)
            row.add_prefix(Gtk.Image.new_from_icon_name("avatar-default-symbolic"))
            accounts.add(row)
        page.add(accounts)

        if error:
            page.add(self._partial_error_group())
        return page

    def _area_error_page(self, area: str, _detail: str | None) -> Adw.StatusPage:
        title_key = {
            "localization": "error.area.localization",
            "hardware": "error.area.hardware",
            "people": "error.area.people",
        }[area]
        page = Adw.StatusPage(
            title=self.text(title_key),
            description=self.text("error.area.body"),
            icon_name="dialog-warning-symbolic",
        )
        retry = Gtk.Button(label=self.text("action.refresh"))
        retry.connect("clicked", lambda _button: self._load_state())
        page.set_child(retry)
        return page


class KorunixApplication(Adw.Application):
    def __init__(self) -> None:
        super().__init__(
            application_id=APPLICATION_ID,
            flags=Gio.ApplicationFlags.DEFAULT_FLAGS,
        )

    def do_activate(self) -> None:
        window = self.get_active_window()
        if window is None:
            window = KorunixWindow(self)
        window.present()


def main() -> int:
    Adw.init()
    application = KorunixApplication()
    return application.run(sys.argv)


if __name__ == "__main__":
    raise SystemExit(main())
