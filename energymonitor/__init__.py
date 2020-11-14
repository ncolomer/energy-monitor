try:
    from importlib import metadata
except ImportError:
    import importlib_metadata as metadata
VERSION = metadata.version('energy-monitor')
