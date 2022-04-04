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

mkdir $WORKBENCH/ghsamples
cd $WORKBENCH/ghsamples

# git clone https://github.com/mdn/web-tech-games.git
# git clone https://github.com/mdn/learning-area.git
# git clone https://github.com/mdn/css-examples.git
# git clone https://github.com/mdn/imsc.git
# git clone https://github.com/mdn/canvas-raycaster.git
# git clone https://github.com/mdn/dom-examples.git
# git clone https://github.com/mdn/webgl-examples.git
# git clone https://github.com/mdn/html-examples.git

cd $WORKBENCH/content
export REV=$(git rev-parse --short HEAD)

cd $WORKBENCH
curl -O $UPDATE_URL/update.json
if [ -f "update.json" ]
then
	export LATEST=$(jq -r -c '.latest' update.json)
	if [ $LATEST == $REV]
	then
		echo "Bundle already exsits for $REV"
		exit 0
	fi
fi

cd $WORKBENCH
export CONTENT_ROOT=$WORKBENCH/content
export BUILD_OUT_ROOT=$WORKBENCH/build
export BUILD_LIVE_SAMPLES_BASE_URL="https://yari-demos.prod.mdn.mozit.cloud"
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
if [ -f "update.json" ]
then
	for OLD_REV in $(jq -r -c '.updates[]' update.json)
	do
		curl -O $UPDATE_URL/packages/$OLD_REV-checksums.zip
	done
	curl -O $UPDATE_URL/packages/$LATEST-checksums.zip
fi

differy package $BUILD_OUT_ROOT --rev $REV

# Note this will fail if any individual file in the input .zip is OVER 1MB. 
zipsplit -n 1048576 ${REV}-content.zip
# Zipsplit only takes the first 4 chars of name as prefix. Replace with full REV.
mmv `echo ${REV} | cut -c 1-4`\* ${REV}\#1
# Files to Json array.
FILES=`(ls -d -- *[0-9].zip | jq -R -s -c 'split("\n")[:-1]')`
# Create update index.json
echo "{\"root\": \"${REV}\", \"files\": ${FILES}}" > index.json

cp update.json ${REV}-update.json
cp ${REV}-content.json content.json

aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.zip"
aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.json"
aws s3 cp update.json s3://${BUCKET}/
aws s3 cp content.json s3://${BUCKET}/
aws s3 cp index.json s3://${BUCKET}/packages/${REV}-content/
aws s3 cp . s3://${BUCKET}/packages/${REV}-content/ --recursive --exclude "*" --include "${REV}_*.zip"