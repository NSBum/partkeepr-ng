// SPDX-License-Identifier: GPL-3.0-or-later
"use strict";

function apiKeyDate(value) {
    return value ? new Date(value + "Z").toLocaleString() : "—";
}

async function openApiKeys() {
    const previous = $$("pk-account-dialog");
    if (previous) previous.destructor();
    const userAtOpen = currentUser;
    let keys, owners;
    try {
        [keys, owners] = await Promise.all([
            api.accountRequest("api-keys"), api.accountRequest("api-key-owners"),
        ]);
    } catch (e) {
        webix.message({ type: "error", text: escapeHtml(e.message) });
        return;
    }
    if (currentUser !== userAtOpen) return;
    const win = webix.ui({
        view: "window", id: "pk-account-dialog", width: Math.min(880, innerWidth - 24),
        height: Math.min(610, innerHeight - 24), position: "center", modal: true,
        head: { view: "toolbar", cols: [
            { view: "label", label: "API keys" },
            { view: "icon", icon: "wxi-close", tooltip: "Close", click: () => win.destructor() },
        ] },
        body: { rows: [
            { view: "template", height: currentUser.is_admin ? 66 : 52, borderless: true,
              template: "Use keys for scripts and integrations. Full keys are shown only once. " +
                "Revocation takes effect on the next request." +
                (currentUser.is_admin ? " As an administrator, you can manage keys for all accounts." : "") },
            { view: "form", id: "pk-api-key-create-form", padding: 10,
              elementsConfig: { labelPosition: "top" }, elements: [
                { cols: [
                    { view: "text", name: "name", label: "Key name", placeholder: "e.g. Stock reporting script" },
                    { view: "richselect", name: "access", label: "Access", width: 150, value: "read", options: [
                        { id: "read", value: "Read-only" }, { id: "read_write", value: "Read / write" },
                    ] },
                    { view: "richselect", name: "expires_in_days", label: "Expires after", width: 140, value: 90, options: [
                        { id: 30, value: "30 days" }, { id: 90, value: "90 days" },
                        { id: 365, value: "1 year" }, { id: "never", value: "Never" },
                    ] },
                ] },
                { cols: [
                    { view: "richselect", name: "user_id", label: "Assigned account", value: currentUser.user_id,
                      hidden: !currentUser.is_admin, options: owners.map(u => ({ id: u.user_id, value: u.username })) },
                    {},
                    { view: "button", id: "pk-api-key-create", value: "Create key", width: 120, css: "webix_primary",
                      click: () => createApiKey(owners) },
                ] },
            ] },
            { view: "form", id: "pk-api-key-secret", hidden: true, padding: 10, elements: [
                { view: "template", height: 44, borderless: true,
                  template: "Copy this key now and store it securely. You cannot view it again after closing this window." },
                { cols: [
                    { view: "text", id: "pk-api-key-secret-value", readonly: true, label: "New key", labelWidth: 65,
                      on: { onFocus: function () { this.getInputNode().select(); } } },
                    { view: "button", id: "pk-api-key-copy", value: "Copy", width: 75,
                      click: async () => {
                          const field = $$("pk-api-key-secret-value");
                          try {
                              await navigator.clipboard.writeText(field.getValue());
                              webix.message("Key copied.");
                          } catch (_) {
                              field.focus(); field.getInputNode().select();
                              webix.message("Press Ctrl+C (or Command+C) to copy the selected key.");
                          }
                      } },
                ] },
            ] },
            { view: "datatable", id: "pk-api-keys-grid", select: "row", rowHeight: 36,
              scroll: "xy", data: keys, columns: [
                { id: "name", header: "Name", fillspace: true, minWidth: 150, template: row => escapeHtml(row.name) },
                { id: "username", header: "Account", width: 120, hidden: !currentUser.is_admin, template: row => escapeHtml(row.username) },
                { id: "key_prefix", header: "Prefix", width: 175, template: row => escapeHtml(row.key_prefix) + "…" },
                { id: "access", header: "Access", width: 100, template: row => row.access === "read" ? "Read-only" : "Read / write" },
                { id: "status", header: "Status", width: 90, template: row => row.revoked_at ? "Revoked" : row.expired ? "Expired" : "Active" },
                { id: "created_at", header: "Created", width: 175, template: row => apiKeyDate(row.created_at) },
                { id: "expires_at", header: "Expires", width: 175, template: row => row.expires_at ? apiKeyDate(row.expires_at) : "Never" },
                { id: "last_used_at", header: "Last used", width: 175, template: row => row.last_used_at ? apiKeyDate(row.last_used_at) : "Not used yet" },
              ] },
            { view: "toolbar", cols: [
                { view: "button", id: "pk-api-key-rename", value: "Rename", width: 95, click: renameApiKey },
                { view: "button", id: "pk-api-key-revoke", value: "Revoke", width: 95, css: "webix_danger", click: revokeApiKey },
                {},
                { view: "button", value: "Refresh", width: 90, click: refreshApiKeys },
                { view: "button", value: "Close", width: 90, click: () => win.destructor() },
            ] },
        ] },
    });
    win.show();
}

async function refreshApiKeys() {
    try {
        const keys = await api.accountRequest("api-keys");
        const grid = $$("pk-api-keys-grid");
        if (grid) { grid.clearAll(); grid.parse(keys); }
    } catch (e) { webix.message({ type: "error", text: escapeHtml(e.message) }); }
}

async function createApiKey(owners) {
    const form = $$("pk-api-key-create-form");
    const values = form.getValues();
    if (!values.name.trim() || [...values.name.trim()].length > 100) {
        return webix.message({ type: "error", text: "Use a key name of 1–100 characters." });
    }
    const win = $$("pk-account-dialog");
    const button = $$("pk-api-key-create");
    // Keep each newly created secret visible until the user explicitly closes
    // the window; reopening the manager permits creating another key.
    button.disable();
    const owner = Number(values.user_id || currentUser.user_id);
    const ownerName = owners.find(u => u.user_id === owner)?.username || currentUser.username;
    try {
        if (values.access === "read_write") {
            await webix.confirm({ title: "Read / write API key", text: "This key will be able to modify inventory as " + escapeHtml(ownerName) + ". Create it?" });
        }
        const result = await api.accountRequest("api-keys", "POST", {
            name: values.name.trim(), access: values.access, user_id: owner,
            expires_in_days: values.expires_in_days === "never" ? null : Number(values.expires_in_days),
        });
        if ($$("pk-account-dialog") !== win) {
            webix.message({ type: "error", text: "The key was created but its window was closed. Revoke it and create a new key to see the full value." });
            return;
        }
        form.hide();
        $$("pk-api-key-secret-value").setValue(result.api_key);
        $$("pk-api-key-secret").show();
        $$("pk-api-key-secret-value").focus();
        $$("pk-api-keys-grid").add(result.key, 0);
        $$("pk-api-keys-grid").select(result.key.id);
    } catch (e) {
        if (e && e.message) webix.message({ type: "error", text: escapeHtml(e.message) });
        if ($$("pk-api-key-create") === button) button.enable();
    }
}

function selectedApiKey() {
    const grid = $$("pk-api-keys-grid");
    const id = grid && grid.getSelectedId();
    if (!id) { webix.message("Select a key first."); return null; }
    return grid.getItem(id);
}

async function renameApiKey() {
    const key = selectedApiKey();
    if (!key) return;
    try {
        const name = await webix.prompt({ title: "Rename API key", input: { value: key.name }, text: "Key name" });
        await api.accountRequest("api-keys/" + key.id, "PATCH", { name });
        await refreshApiKeys();
    } catch (e) { if (e && e.message) webix.message({ type: "error", text: escapeHtml(e.message) }); }
}

async function revokeApiKey() {
    const key = selectedApiKey();
    if (!key) return;
    if (key.revoked_at) return webix.message("This key is already revoked.");
    try {
        await webix.confirm({ title: "Revoke API key", ok: "Revoke", text: "Revoke “" + escapeHtml(key.name) + "”? Integrations using it will stop working." });
        await api.accountRequest("api-keys/" + key.id, "DELETE");
        const secret = $$("pk-api-key-secret-value");
        if (secret && secret.getValue().startsWith(key.key_prefix)) {
            secret.setValue(""); $$("pk-api-key-secret").hide();
        }
        await refreshApiKeys();
    } catch (e) { if (e && e.message) webix.message({ type: "error", text: escapeHtml(e.message) }); }
}
