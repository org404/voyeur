
echo
curl http://localhost:8000/api/v1/health

echo
curl -H "X-Namespace: This sample sentence was crafted to be exactly 65 characters long" http://localhost:8000/api/v1/logs
curl "http://localhost:8000/api/v1/logs?namespace=test_value"

echo
curl "http://localhost:8000/api/v1/logs?namespace=test_value&page=0"
curl "http://localhost:8000/api/v1/logs?namespace=test_value&page=1&page_size=10"
curl -H "X-Page-Size: 100" "http://localhost:8000/api/v1/logs?namespace=test_value&page=2"
 
echo
sample="This is test data!"
curl -X POST -d "{\"namespace\": \"fuzzer1\", \"content\": \"$sample\"}" -H "Content-Type: application/json" http://localhost:8000/api/v1/logs

sample="This is second test data!"
curl -X POST -d "{\"namespace\": \"fuzzer1\", \"content\": \"$sample\"}" -H "Content-Type: application/json" http://localhost:8000/api/v1/logs

echo
curl "http://localhost:8000/api/v1/logs?namespace=fuzzer1"

echo
sample1="Super cool value one."
sample2="Berry cool value two."
sample3='{"x": 123, "some_key": "some_data"}'

data="[{\"namespace\": \"fuzzer69\", \"content\": \"$sample1\"},{\"namespace\": \"fuzzer69\", \"content\": \"$sample2\"},{\"namespace\": \"fuzzer69\", \"content\": $sample3}]"
curl -X POST -d "$data" -H "Content-Type: application/json" http://localhost:8000/api/v1/logs

echo
sample="This is test data to update!"
curl -X PUT -d "{\"namespace\": \"fuzzer1\", \"content\": \"$sample\"}" -H "Content-Type: application/json" http://localhost:8000/api/v1/logs/3
curl -X PUT -d "{\"namespace\": \"fuzzer1\", \"content\": \"$sample\"}" -H "Content-Type: application/json" http://localhost:8000/api/v1/logs/4

echo
curl -X DELETE "http://localhost:8000/api/v1/logs?namespace=fuzzer1"
curl -X DELETE -H "X-Namespace: fuzzer69" http://localhost:8000/api/v1/logs

