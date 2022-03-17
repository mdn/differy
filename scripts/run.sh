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
export LATEST=$(jq -r -c '.latest' update.json)
if [ $LATEST == $REV]
then
	echo "Bundle already exsits for $REV"
	exit 0
fi

cd $WORKBENCH
export CONTENT_ROOT=$WORKBENCH/content
export BUILD_OUT_ROOT=$WORKBENCH/build
export BUILD_LIVE_SAMPLES_BASE_URL="https://yari-demos.prod.mdn.mozit.cloud"
mkdir -p $BUILD_OUT_ROOT

cd $WORKBENCH/yari

export BUILD_LIVE_SAMPLES_BASE_URL=https://yari-demos.stage.mdn.mozit.cloud
export BUILD_INTERACTIVE_EXAMPLES_BASE_URL=https://interactive-examples.stage.mdn.mozilla.net
export BUILD_FLAW_LEVELS="*:ignore"
export REACT_APP_ENABLE_PLUS=true
export REACT_APP_DISABLE_AUTH=false
export REACT_APP_INTERACTIVE_EXAMPLES_BASE_URL=https://interactive-examples.stage.mdn.mozilla.net
export REACT_APP_FXA_SIGNIN_URL=/users/fxa/login/authenticate/
export REACT_APP_FXA_SETTINGS_URL=https://accounts.stage.mozaws.net/settings/
export REACT_APP_MDN_PLUS_SUBSCRIBE_URL=https://accounts.stage.mozaws.net/subscriptions/products/prod_Jtbg9tyGyLRuB0
export REACT_APP_MDN_PLUS_5M_PLAN=price_1JFoTYKb9q6OnNsLalexa03p
export REACT_APP_MDN_PLUS_5Y_PLAN=price_1JpIPwKb9q6OnNsLJLsIqMp7
export REACT_APP_MDN_PLUS_10M_PLAN=price_1K6X7gKb9q6OnNsLi44HdLcC
export REACT_APP_MDN_PLUS_10Y_PLAN=price_1K6X8VKb9q6OnNsLFlUcEiu4

yarn
yarn prepare-build
yarn build -n
cp -r client/build/* $BUILD_OUT_ROOT/

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
cp ${REV}-content.json content.json

aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.zip"
aws s3 cp . s3://${BUCKET}/packages/ --recursive --exclude "*" --include "${REV}-*.json"
aws s3 cp update.json s3://${BUCKET}/
aws s3 cp content.json s3://${BUCKET}/