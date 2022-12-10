# Releasing

Before releasing, a few things need to happen in order to sucessfully release
and publish the crates.

## Updating Changelog

First make sure the [changelog](./CHANGELOG.md) is updated.

## Versioning

Make sure that all the crates have the same version.
This version will be used to create the git tag.

## Checking

Check the crates before creating tag and publishing.

```
python3 .github/workflows/publish.py check -v
python3 .github/workflows/publish.py publish --dry-run -v
```

## Tagging

Create a git tag and make sure the tag starts with `v` and follows the version. like:

```
git tag -s -a v1.2.3 -m "Version 1.2.3"
git push origin v1.2.3
```

## Publishing

Pushing a tag into gitlab repository will automatically
publish the crates into [crates.io](https://crates.io/search?q=kelk).

## Bumping version

After publishing the crates, the version should update.
Update the version inside the `Cargo.toml` files by increasing the minor version of the crate.
For example from `0.0.0` to `0.1.0`

Create a commit and push it to main branch:
```
git commit -m "Bumping version to 0.1.0"
git push origin HEAD
```

Please make sure you update the [changelog](./CHANGELOG.md) file as well.
