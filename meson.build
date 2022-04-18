project('mv-mqtt-client',
        'rust',
        version: '0.0.2')

gnome = import('gnome')

application_id = 'io.github.jgreitemann.mv-mqtt-client'

dependency('glib-2.0', version: '>= 2.56')
dependency('gio-2.0', version: '>= 2.56')
dependency('gdk-pixbuf-2.0')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>= 1.1.0')

cargo = find_program('cargo', required: true)
find_program('glib-compile-resources', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()

profile = get_option('profile')

subdir('data')
subdir('src')
