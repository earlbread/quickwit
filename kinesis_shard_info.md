# Kinesis 샤드 ID 확인 방법

## 1. AWS Management Console

1. **Kinesis 콘솔 접속**
   - https://console.aws.amazon.com/kinesis
   - 해당 리전 선택

2. **Data streams 선택**
   - 좌측 메뉴에서 "Data streams" 클릭
   - 확인하려는 스트림 이름 클릭

3. **Shards 탭 확인**
   - 스트림 상세 페이지에서 "Shards" 탭 클릭
   - 여기서 모든 샤드 정보 확인 가능:
     - Shard ID (예: shardId-000000000000)
     - Status (OPEN/CLOSED)
     - Parent shard ID
     - Hash key range
     - Sequence number range

## 2. AWS CLI 명령어

### 기본 샤드 목록 확인
```bash
aws kinesis list-shards --stream-name <stream-name>
```

### 특정 필드만 추출
```bash
# 샤드 ID만 추출
aws kinesis list-shards --stream-name <stream-name> \
  --query 'Shards[*].ShardId' --output text

# 샤드 ID와 상태 확인
aws kinesis list-shards --stream-name <stream-name> \
  --query 'Shards[*].[ShardId, SequenceNumberRange.EndingSequenceNumber]' \
  --output table
```

### 스트림 상세 정보 (구버전 API)
```bash
aws kinesis describe-stream --stream-name <stream-name>
```

## 3. boto3 (Python)로 확인

```python
import boto3

kinesis = boto3.client('kinesis', region_name='us-east-1')

# 샤드 목록 가져오기
response = kinesis.list_shards(StreamName='your-stream-name')

for shard in response['Shards']:
    shard_id = shard['ShardId']
    parent_id = shard.get('ParentShardId', 'None')
    
    # 샤드가 닫혔는지 확인
    is_closed = 'EndingSequenceNumber' in shard['SequenceNumberRange']
    status = 'CLOSED' if is_closed else 'OPEN'
    
    print(f"Shard ID: {shard_id}")
    print(f"  Status: {status}")
    print(f"  Parent: {parent_id}")
    print(f"  Hash Range: {shard['HashKeyRange']['StartingHashKey']} - {shard['HashKeyRange']['EndingHashKey']}")
    print()
```

## 4. 샤드 ID 형식

Kinesis 샤드 ID는 다음과 같은 형식을 따릅니다:
- **형식**: `shardId-000000000000`
- **특징**: 
  - 12자리 숫자로 구성
  - 0부터 시작하여 순차적으로 증가
  - 리샤딩 시 새로운 ID 할당

## 5. 샤드 상태 확인 시 주의사항

### 활성(OPEN) 샤드
- `EndingSequenceNumber`가 없음
- 데이터 읽기/쓰기 가능

### 닫힌(CLOSED) 샤드
- `EndingSequenceNumber`가 존재
- 리샤딩(분할/병합) 후 발생
- 읽기만 가능, 쓰기 불가

### 부모-자식 관계
- **분할(Split)**: 1개 부모 → 2개 자식
  - `ParentShardId` 필드에 부모 샤드 ID 저장
- **병합(Merge)**: 2개 부모 → 1개 자식
  - `ParentShardId`와 `AdjacentParentShardId`에 두 부모 샤드 ID 저장

## 6. 리샤딩 이벤트 감지

```bash
# 시간대별 샤드 변경 모니터링
watch -n 60 'aws kinesis list-shards --stream-name <stream-name> \
  --query "Shards[*].[ShardId, SequenceNumberRange.EndingSequenceNumber]" \
  --output table'
```

## 7. Quickwit 로그에서 확인

Quickwit 로그에서도 샤드 정보를 확인할 수 있습니다:

```bash
# Quickwit 로그에서 샤드 할당 정보 확인
grep "assigned_shards" quickwit.log

# 예시 출력:
# Starting Kinesis source. stream_name=cloudfront-analytics assigned_shards=shardId-000000000000, shardId-000000000001
```