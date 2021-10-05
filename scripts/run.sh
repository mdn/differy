#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -x

mkdir -p workbench
export WORKBENCH=$(realpath workbench)
if [ ! -d $WORKBENCH/.bin ]; then mkdir $WORKBENCH/.bin; fi
export PATH=$WORKBENCH/.bin:$PATH

curl -L https://github.com/mdn/differy/releases/latest/download/differy-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C $WORKBENCH/.bin/

cd $WORKBENCH

git clone https://github.com/mdn/yari.git
git clone https://github.com/mdn/content.git
git clone https://github.com/mdn/interactive-examples.git

cd $WORKBENCH/content
export REV=$(git rev-parse --short HEAD)

cd $WORKBENCH
curl -O $UPDATE_URL/update.json
export LATEST=$(jq -r -c '.latest' update.json)
if [ $LATEST == $REV]
then
	echo "Bundle already exsits for $REV"
	exit 0
fi

cd $WORKBENCH
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

cd $WORKBENCH
for OLD_REV in $(jq -r -c '.updates[]' update.json)
do
	curl -O $UPDATE_URL/packages/$OLD_REV-checksums.zip
done
curl -O $UPDATE_URL/packages/$LATEST-checksums.zip

differy package $BUILD_OUT_ROOT --rev $REV
cp update.json ${REV}-update.json

aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.zip"
aws s3 cp ${REV}-update.json s3://${BUCKET}/packages/
aws s3 cp update.json s3://${BUCKET}/