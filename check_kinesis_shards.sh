#!/bin/bash

# Kinesis 샤드 정보 확인 스크립트

STREAM_NAME="cloudfront-analytics"
REGION="ap-northeast-2"  # 또는 해당 리전

echo "=== Kinesis 샤드 정보 확인 ==="
echo ""

# 1. 스트림 요약 정보 확인
echo "1. 스트림 요약 정보:"
aws kinesis describe-stream-summary \
    --stream-name $STREAM_NAME \
    --region $REGION \
    --output json | jq '.'

echo ""
echo "2. 모든 샤드 목록 (상태 포함):"
# 2. 모든 샤드 목록 확인 (닫힌 샤드 포함)
aws kinesis list-shards \
    --stream-name $STREAM_NAME \
    --region $REGION \
    --output json | jq '.Shards[] | {
        ShardId: .ShardId,
        Status: (if .SequenceNumberRange.EndingSequenceNumber then "CLOSED" else "OPEN" end),
        ParentShardId: .ParentShardId,
        AdjacentParentShardId: .AdjacentParentShardId,
        StartingHashKey: .HashKeyRange.StartingHashKey,
        EndingHashKey: .HashKeyRange.EndingHashKey,
        StartingSequenceNumber: .SequenceNumberRange.StartingSequenceNumber,
        EndingSequenceNumber: .SequenceNumberRange.EndingSequenceNumber
    }'

echo ""
echo "3. 활성(OPEN) 샤드만 확인:"
# 3. 활성 샤드만 필터링
aws kinesis list-shards \
    --stream-name $STREAM_NAME \
    --region $REGION \
    --output json | jq '.Shards[] | select(.SequenceNumberRange.EndingSequenceNumber == null) | {
        ShardId: .ShardId,
        Status: "OPEN",
        ParentShardId: .ParentShardId
    }'

echo ""
echo "4. 샤드 계보 (부모-자식 관계):"
# 4. 샤드 부모-자식 관계 확인
aws kinesis list-shards \
    --stream-name $STREAM_NAME \
    --region $REGION \
    --output json | jq '.Shards[] | select(.ParentShardId != null or .AdjacentParentShardId != null) | {
        ShardId: .ShardId,
        ParentShardId: .ParentShardId,
        AdjacentParentShardId: .AdjacentParentShardId,
        IsClosed: (if .SequenceNumberRange.EndingSequenceNumber then true else false end)
    }'

echo ""
echo "5. 샤드 메트릭 (CloudWatch):"
# 5. CloudWatch에서 샤드별 메트릭 확인
aws cloudwatch list-metrics \
    --namespace AWS/Kinesis \
    --metric-name IncomingRecords \
    --dimensions Name=StreamName,Value=$STREAM_NAME \
    --region $REGION \
    --output json | jq '.Metrics[].Dimensions[] | select(.Name == "ShardId") | .Value' | sort | uniq
