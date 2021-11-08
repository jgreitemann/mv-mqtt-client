sources = files(
  'bin/main.rs',
  'bin/app.rs',
  'bin/app/app_ctrl.rs',
  'bin/app/client.rs',
  'bin/app/helpers.rs',
  'bin/app/macros.rs',
  'mvjson/lib.rs',
  'mvjson/definitions.rs',
  'mvjson/helpers.rs',
)

cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    profile,
  ]
)
