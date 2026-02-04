`OpenMoji-flags.woff2` is a subset of `OpenMoji-color-glyf_colr_0.woff2` [OpenMoji](https://github.com/hfg-gmuend/openmoji/) font that only contains the emoji flag symbols.

The subset was created using [pyftsubset](https://fonttools.readthedocs.io/en/stable/subset/):

```sh
pyftsubset ./OpenMoji-color-glyf_colr_0.woff2 --unicodes=1F1E6-1F1FF --output-file=OpenMoji-flags.woff2 --flavor=woff2
```
