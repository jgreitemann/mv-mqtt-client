project('mv-mqtt-client',
        'rust',
        version: '0.0.2')

gnome = import('gnome')

dependency('glib-2.0', version: '>= 2.56')
dependency('gio-2.0', version: '>= 2.56')
dependency('gdk-pixbuf-2.0')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>= 1.1.0')

find_program('cargo', required: true)
find_program('glib-compile-resources', required: true)

cargo_script = find_program('build-aux/cargo.sh')

version = meson.project_version()
version_array = version.split('.')
major_version = version_array[0].to_int()
minor_version = version_array[1].to_int()
version_micro = version_array[2].to_int()

profile = get_option('profile')

subdir('data')
subdir('src')
