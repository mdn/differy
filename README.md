# Differy - MDN's partial update bundler

Due to the nature of [yari] we don't know which articles changed since out last build.
The kuma script in an unmodified file might render different that it did last time.
At some point we should solve this in [yari], but now for now.

## Why

We need content bundles for the offline capability of the MDN apps. Further, we need
daily updates to deliver new content without downloading the huge content bundle.
**Differy** creates these bundles and a GitHub action uploads them for distribution.

## Basic Usage and Example

**Differy** generates updates based on a list of hashes of all files generated in previous
builds. As input we additionally need a fresh build of [mdn/content] and
[mdn/interactive-examples] and the short-rev of the [mdn/content]

```sh
differy package $BUILD_OUT_ROOT --rev  $(cd $CONTENT; git rev-parse --short HEAD)
```

On top of that we want a reference "update.json" and the checksum archives for
all version we want to generate updates for.

### Output

The `$BUILD_OUT_ROOT` directory should contain the artifacts of:

- `yarn prepare-build && yarn build -n` from [mdn/content]
- `yarn build` from [mdn/interactive-examples]

**Differy** generates:

- `xxxxxxxxx-content.zip`: All of `$BUILD_OUT_ROOT`
- `xxxxxxxxx-app-content.zip`: All of `$BUILD_OUT_ROOT`
- `xxxxxxxxx-content.json`: a json file containing the names of all
  content files.
  with modified links to _interactive-examples_
- `xxxxxxxxx-yyyyyyyyy-update.zip`: the changed files between `xxxxxxxxx` and
  `yyyyyyyyy` plus a file called `removed` containing list of all files that
  have been removed
- `xxxxxxxxx-yyyyyyyyy-app-update.zip`: the same with modified links
- `xxxxxxxxx-yyyyyyyyy-diff.json`: a json file containing the names of changed
  files between `xxxxxxxxx` and `yyyyyyyyy`
  have been removed
- `update.json` the modified input file

### Example

Assume we have the flowing scenario:

```sh
> ls -1 $(pwd)
3bfe5e8ee-checksums.zip
723965504-checksums.zip
update.json

> cat update.json
{
  "date": "2021-08-20T13:43:20.024561",
  "latest": "3bfe5e8ee",
  "updates": [
    "723965504"
  ]
}

> echo $(cd $CONTENT; git rev-parse --short HEAD)
c4123a3f1
```

We now run:

```sh
> differy package $BUILD_OUT_ROOT --rev c4123a3f1
packaging update c4123a3f1 → 3bfe5e8ee
packaging update c4123a3f1 → 723965504
building content for c4123a3f1
```

This will generate:

```sh
> ls -1 {c4123a3f1*,update.json}

c4123a3f1-3bfe5e8ee-app-update.zip
c4123a3f1-3bfe5e8ee-diff.json
c4123a3f1-3bfe5e8ee-update.zip
c4123a3f1-723965504-app-update.zip
c4123a3f1-723965504-diff.json
c4123a3f1-723965504-update.zip
c4123a3f1-app-content.zip
c4123a3f1-checksums.zip
c4123a3f1-content.zip
c4123a3f1-content.json
update.json
```

## Automating and Uploading Artifacts

We include a shell script that automates everything we need to generate
the latest bundles. And then uploads it to S3 for distribution.

Take a look at [scripts/run.sh](scripts/run.sh)

[yari]: https://github.com/mdn/yari
[mdn/content]: https://github.com/mdn/content
[mdn/interactive-examples]: https://github.com/mdn/interactive-examples
