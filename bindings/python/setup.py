from setuptools import setup

def build_native(spec):
    # build an example rust library
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='../c/'
    )
    build = spec.add_external_build(
        cmd=['strip', 'target/release/liblomak.so'],
        path='../c/'
    )

    spec.add_cffi_module(
        module_path='lomak._native',
        dylib=lambda: build.find_dylib('lomak', in_path='target/release'),
        header_filename=lambda: build.find_header('lomak.h', in_path='target'),
        rtld_flags=['NOW', 'NODELETE']
    )

setup(
    name='lomak',
    version='0.1',
    packages = ['lomak'],
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['milksnake'],
    milksnake_tasks=[
        build_native
    ]
)

