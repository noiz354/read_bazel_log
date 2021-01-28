#!/bin/bash

#measurement=tableName
measurement=$1
#in format: anyKey=anyValue,... Example:id=sonoda-bot
tagMap=$2
#in format: value=abc,anyfieldKey=anyValue;... Example:value=100,reset=50
valueMap=$3

echo "$measurement"
# shellcheck disable=SC2001
tagTrimSpace=$(echo "$tagMap" | sed 's/ //g')
echo "tag : $tagTrimSpace"
# shellcheck disable=SC2116
valueTrimSpace=$(echo "${valueMap//[$'\t\r\n ']}")
echo "value : $valueTrimSpace"

curl -H "Authorization: Token co7koZRr209kbA2MDeejKdr7DyDSF4HwbNEbjEUNm8gxAPN2nJ_510FfnX9KNhZgtMLOQbCBtlxKgo3vn2cwyg==" -i -XPOST 'http://127.0.0.1:8086/write?db=test' --data-binary "$measurement,$tagTrimSpace $valueTrimSpace"
