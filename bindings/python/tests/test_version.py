class TestVersion:
    def test_version(self):
        from cnvx import __version__

        assert __version__
