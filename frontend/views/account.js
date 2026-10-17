// SPDX-License-Identifier: GPL-3.0-or-later
// Account menu and self-service dialogs. No password is stored in browser storage.
"use strict";

let accountPreferences = { start_section: "categories", parts_limit: 200 };

function closeAccountViews() {
    for (const id of ["pk-account-menu", "pk-account-dialog"]) {
        const view = $$(id);
        if (view) view.destructor();
    }
}

function openAccountMenu(anchor) {
    let menu = $$("pk-account-menu");
    if (!menu) {
        menu = webix.ui({
            view: "popup", id: "pk-account-menu", width: 205,
            body: {
                view: "menu", id: "pk-account-menu-list", layout: "y", autoheight: true,
                template: "#value#",
                data: [
                    { id: "profile", value: "My profile" },
                    { id: "password", value: "Change password" },
                    { id: "preferences", value: "Preferences" },
                    { id: "api-keys", value: "API keys" },
                    { id: "logout", value: "Sign out" },
                ],
                on: { onMenuItemClick: function (id) {
                    menu.hide();
                    if (id === "logout") doLogout();
                    else if (id === "api-keys") openApiKeys();
                    else openAccountDialog(id);
                } },
            },
        });
    }
    if (menu.isVisible()) menu.hide();
    else {
        menu.show(anchor.$view, { pos: "bottom" });
        webix.UIManager.setFocus($$("pk-account-menu-list"));
    }
}

async function openAccountDialog(kind) {
    const previous = $$("pk-account-dialog");
    if (previous) previous.destructor();
    const userAtOpen = currentUser;
    let values = {};
    try {
        if (kind !== "password") values = await api.accountRequest(kind);
    } catch (e) {
        webix.message({ type: "error", text: escapeHtml(e.message) });
        return;
    }
    if (!currentUser || currentUser !== userAtOpen) return;
    const titles = { profile: "My profile", password: "Change password", preferences: "Preferences" };
    let fields;
    if (kind === "profile") {
        fields = [
            { view: "text", name: "username", label: "Username", readonly: true },
            { view: "text", name: "email", label: "Email", attributes: { type: "email", autocomplete: "email" } },
            { view: "text", name: "role", label: "Role", readonly: true },
        ];
        values.role = values.is_admin ? "Administrator" : "User";
    } else if (kind === "password") {
        fields = [
            { view: "text", type: "password", name: "current_password", label: "Current password", attributes: { autocomplete: "current-password" } },
            { view: "text", type: "password", name: "new_password", label: "New password (at least 12 characters)", attributes: { autocomplete: "new-password" } },
            { view: "text", type: "password", name: "confirm_password", label: "Confirm new password", attributes: { autocomplete: "new-password" } },
            { view: "template", borderless: true, height: 52,
              template: "After changing your password, sign in again. Other sessions will also be signed out." },
        ];
    } else {
        fields = [
            { view: "richselect", name: "start_section", label: "Open at sign-in", options: [
                { id: "categories", value: "Parts / Categories" },
                { id: "storage", value: "Storage" },
                { id: "footprints", value: "Footprints" },
                { id: "projects", value: "Projects" },
            ] },
            { view: "richselect", name: "parts_limit", label: "Maximum parts shown", options: [
                { id: 50, value: "50" }, { id: 100, value: "100" }, { id: 200, value: "200" },
            ] },
            { view: "template", borderless: true, height: 52,
              template: "Saved to your account across browsers. Use filters to find parts beyond the display limit." },
        ];
    }
    const win = webix.ui({
        view: "window", id: "pk-account-dialog", width: Math.min(460, window.innerWidth - 24),
        position: "center", modal: true,
        head: { view: "toolbar", cols: [
            { view: "label", label: titles[kind] },
            { view: "icon", icon: "wxi-close", tooltip: "Close", click: () => win.destructor() },
        ] },
        body: {
            view: "form", id: "pk-account-form", elementsConfig: { labelPosition: "top" },
            elements: [...fields, { cols: [
                {},
                { view: "button", value: "Cancel", width: 90, click: () => win.destructor() },
                { view: "button", id: "pk-account-save", value: kind === "password" ? "Change password" : "Save",
                  width: kind === "password" ? 150 : 90, css: "webix_primary",
                  click: () => saveAccountDialog(kind, win) },
            ] }],
        },
    });
    $$("pk-account-form").setValues(values);
    win.show();
    $$("pk-account-form").focus(kind === "profile" ? "email" : kind === "password" ? "current_password" : "start_section");
}

async function saveAccountDialog(kind, win) {
    const data = $$("pk-account-form").getValues();
    let body;
    if (kind === "password") {
        if (!data.current_password) return webix.message({ type: "error", text: "Enter your current password." });
        if ([...data.new_password].length < 12) return webix.message({ type: "error", text: "Use at least 12 characters for the new password." });
        if (data.new_password !== data.confirm_password) return webix.message({ type: "error", text: "The new passwords do not match." });
        body = { current_password: data.current_password, new_password: data.new_password, confirm_password: data.confirm_password };
    } else if (kind === "profile") {
        body = { email: data.email.trim() };
        if (!webix.rules.isEmail(body.email)) return webix.message({ type: "error", text: "Enter a valid email address." });
    } else {
        body = { start_section: data.start_section, parts_limit: Number(data.parts_limit) };
    }
    const save = $$("pk-account-save");
    save.disable();
    try {
        const result = await api.accountRequest(kind, kind === "password" ? "POST" : "PUT", body);
        if (kind === "preferences") {
            accountPreferences = result;
            await loadParts({});
        }
        if ($$("pk-account-dialog") === win) win.destructor();
        if (kind === "password") {
            mountLogin();
            webix.message({ text: "Password changed. Please sign in again." });
        } else webix.message({ text: kind === "profile" ? "Profile saved." : "Preferences saved." });
    } catch (e) {
        webix.message({ type: "error", text: escapeHtml(e.message) });
    } finally {
        if ($$("pk-account-save") === save) save.enable();
    }
}
