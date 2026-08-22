fn main() {
    // Translations are *bundled* (compiled into the binary) rather than
    // loaded from .mo files at runtime: this same binary ships to iOS,
    // Android and wasm, none of which have a convenient place to install a
    // system gettext locale tree.
    //
    // Catalogs live at lang/<locale>/LC_MESSAGES/<crate>.po, the layout
    // with_bundled_translations expects.
    //
    // The default translation context is the enclosing component's name,
    // which would key the same English string differently depending on
    // which component it appeared in -- so "Payment" inside two components
    // would need translating twice. Turning it off gives one flat
    // namespace where identical source strings share a translation.
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("lang")
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
