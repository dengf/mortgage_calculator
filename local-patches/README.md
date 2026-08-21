# local-patches/

Vendored copies of registry crates carrying a small, targeted local fix,
wired in via the root `Cargo.toml`'s `[patch.crates-io]`. Each subdirectory
is an unmodified copy of the published crate source except for the specific
change noted below — diff against the real registry copy
(`~/.cargo/registry/src/*/<crate>-<version>/`) to see exactly what changed.

## i-slint-backend-winit-1.17.1

One line: `lib.rs`'s `try_create_window_with_fallback_renderer()` has an
explicit type annotation added to its `find_map` closure parameter
(`renderer_factory: fn(&Rc<SharedBackendData>) -> Result<Box<dyn
WinitCompatibleRenderer>, PlatformError>`), matching exactly what rustc's
own `error[E0282]: type annotations needed` diagnostic suggests. Without
it, building this app for iOS fails to compile this crate. Remove this
patch (and the `[patch.crates-io]` entry) once a Slint release upstream
carries an equivalent fix — nothing else in this vendored copy has changed.
