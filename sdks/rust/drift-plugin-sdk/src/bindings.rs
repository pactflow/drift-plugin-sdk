//! Bindings for the `data-values` interface of the WIT in `wit/`.
//!
//! Only `data-values` is generated here, from a world that imports nothing else. A plugin has a
//! world of its own, and a world that exported more interfaces than the plugin does would clash with
//! it when the component is built.
//!
//! `wit/` in this crate is a link to the SDK's vendored WIT, `../wit`, so that the WIT is part of the
//! published crate. `just sdk rust sync-wit` updates it.

wit_bindgen::generate!({
    inline: "
        package drift-plugin-sdk:bindings;

        world data-values {
            import drift:plugins/data-values@1.0.0;
        }
    ",
    path: "wit/plugin.wit",
    generate_all,
    additional_derives: [PartialEq, Clone]
});
