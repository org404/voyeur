: '
docker build --target tester -t api-tester api && docker run --network=host --rm api-tester
'

#: '
echo
curl "http://localhost:8000/api/v1/health" || exit 1

echo
curl -H "X-Namespace: This sample sentence was crafted to be exactly 65 characters long" "http://localhost:8000/api/v1/entries"

echo
curl "http://localhost:8000/api/v1/entries?namespace=test_value&page=0"
curl "http://localhost:8000/api/v1/entries?namespace=test_value&page=1&page_size=10"
curl -H "X-Page-Size: 100" "http://localhost:8000/api/v1/entries?namespace=test_value&page=2"
 
echo
sample="This is test data!"
curl -X POST -d "{\"text\": \"$sample\"}" -H "X-NAMESPACE: fuzzer1" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries"

sample="This is second test data!"
curl -X POST -d "{\"text\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries?namespace=fuzzer1"

echo
curl "http://localhost:8000/api/v1/entries?namespace=fuzzer1&page=0"

echo
sample1="Super cool value one."
sample2="Berry cool value two."
sample3='{"x": 123, "some_key": "some_data"}'

data="[{\"logs\": \"$sample1\"},{\"logs\": \"$sample2\"},{\"logs\": $sample3}]"
curl -X POST -d "$data" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries?namespace=fuzzer69"

echo
sample="This is test data to update!"
curl -X PUT -d "{\"logs\": \"$sample\"}" -H "X-Namespace: fuzzer1" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/3"
curl -X PUT -d "{\"logs\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/4?namespace=fuzzer1"

curl -X PUT -d "{\"logs\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/4294967295?namespace=fuzzer1"
curl -X PUT -d "{\"logs\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/18446744073709551615?namespace=fuzzer1"
curl -sfX PUT -d "{\"logs\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/18446744073709551616?namespace=fuzzer1" || \
    echo -e "expected error ID 18446744073709551616 (u64::MAX + 1)\n"

curl "http://localhost:8000/api/v1/entries?namespace=fuzzer1&page=0"

echo
curl -X PUT -d "{\"logs\": \"$sample\"}" -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/123456789?namespace=test_1"
curl "http://localhost:8000/api/v1/entries/123456789?namespace=test_1"
curl -X DELETE -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/123456789?namespace=test_1"
curl -X DELETE -H "Content-Type: application/json" "http://localhost:8000/api/v1/entries/123456789?namespace=test_1"

echo
curl -X DELETE "http://localhost:8000/api/v1/entries?namespace=fuzzer1"
curl -X DELETE -H "X-Namespace: fuzzer69" "http://localhost:8000/api/v1/entries"
#'
