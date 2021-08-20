#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -x

mkdir -p workbench
export WORKBENCH=$(realpath workbench)
if [ ! -d $WORKBENCH/.bin ]; then mkdir $WORKBENCH/.bin; fi
export PATH=$WORKBENCH/.bin:$PATH

DIFFERY_LATEST=$(curl -sL https://api.github.com/repos/mdn/differy/releases/latest | jq -r ".tag_name")
if ! type "differy" > /dev/null; then
	DIFFERY_CURRENT=0
else
	DIFFERY_CURRENT=$(differy -V | sed 's/differy /v/')
fi

if [ $DIFFERY_LATEST != $DIFFERY_CURRENT ]
then
	curl -L https://github.com/mdn/differy/releases/latest/download/differy-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C $WORKBENCH/.bin/
fi

cd $WORKBENCH

git clone https://github.com/mdn/yari.git
git clone https://github.com/mdn/content.git
git clone https://github.com/mdn/interactive-examples.git

export CONTENT_ROOT=$WORKBENCH/content
export BUILD_OUT_ROOT=$WORKBENCH/build

mkdir -p $BUILD_OUT_ROOT

cd $WORKBENCH/yari

yarn
yarn prepare-build
yarn build -n


cd $WORKBENCH/interactive-examples

yarn
yarn build

mv docs $BUILD_OUT_ROOT/examples

cd $WORKBENCH/content

export REV=$(git rev-parse --short HEAD)

cd $WORKBENCH

curl -O $UPDATE_URL/update.json

for OLD_REV in $(jq -r -c '.updates[]' update.json)
do
	curl -O $UPDATE_URL/packages/$OLD_REV-checksums.zip
done
curl -O $UPDATE_URL/packages/$(jq -r -c '.latest' update.json)-checksums.zip

differy package $BUILD_OUT_ROOT --rev $REV

cp update.json ${REV}-update.json

aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.{json,zip}"
aws s3 cp update.json s3://${BUCKET}/