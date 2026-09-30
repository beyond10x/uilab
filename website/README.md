# uilab documentation site

This Docusaurus site is the public uilab documentation, built for `https://beyond10x.github.io/uilab/`.

```console
cd website
npm ci
npm start          # http://localhost:3000/uilab/
npm run build      # fails on any broken link
```

The screenshots under `static/img/screens/` are taken from `uilab serve` over a copy of
`examples/library`. Publication follows the organization's documentation process; this directory
only builds.
