resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = configure_file(
  input: '@0@.desktop.in'.format(application_id),
  output: '@BASENAME@',
  configuration: desktop_conf,
  install: true,
  install_dir: datadir / 'applications'
)

# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file
    ]
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_file = configure_file(
  input: '@0@.metainfo.xml.in'.format(application_id),
  output: '@BASENAME@',
  configuration: appdata_conf,
  install: true,
  install_dir: datadir / 'metainfo'
)

# Validate Appdata
if appstream_util.found()
  test(
    'validate-appdata', appstream_util,
    args: [
      'validate', '--nonet', appdata_file
    ]
  )
endif


subdir('icons')
