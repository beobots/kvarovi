# Electricity


## Lambda

### Setup lambda role

Execute all of commands from root of project

```bash
# create role
aws iam create-role --role-name electricity_lambda_execution_role --assume-role-policy-document file://electricity/iam/execution-role-policy.json

# attach policy
aws iam attach-role-policy --role-name electricity_lambda_execution_role --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

# create dynamodb policy
POLICY_ARN=$(aws iam create-policy --policy-name electricity_lambda_dynamodb_policy --policy-document file://electricity/iam/dynamodb-policy.json | jq -r .Policy.Arn)

# attach it to above role
aws iam attach-role-policy --role-name electricity_lambda_execution_role --policy-arn $POLICY_ARN

```

### Building Lambda

In this case we are going to use cargo lambda subcommand but with docker image since setting it up is very cumbersome and requires zig for cross-compilation.

```bash
# for prod
#
# there is also option for architecture
# --arm64
# default is x86_64
#

docker run -v "$(pwd)":/code ghcr.io/cargo-lambda/cargo-lambda /bin/bash -c "cd /code && cargo lambda build --features lambda --release --output-format zip"

# debug
docker run -v "$(pwd)":/code ghcr.io/cargo-lambda/cargo-lambda /bin/bash -c "cd /code && cargo lambda build --features lambda --output-format zip"

```

### Deploying

#### Upload via S3 (Optional)

```bash
# create s3
aws s3api create-bucket --bucket beobots-lambda-storage

# upload artifact
aws s3 cp ./target/lambda/electricity/bootstrap.zip s3://beobots-lambda-storage/electricity/bootstrap.zip
```

We grab ARN of the role from the setup step and:

```bash
export ROLE_ARN='REPLACE ME FROM SETUP STEP'

# from file
aws lambda create-function --function-name electricity_data_cron \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/electricity/bootstrap.zip \
  --runtime provided.al2 \
  --role $ROLE_ARN \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
#   --architectures arm64
# if you chose non default (x86_64) architecutre during the build
# don't forget to add \ above if you add this at the bottom

# or from S3 (you will need this for large artifacts: debug builds)
aws lambda create-function --function-name electricity_data_cron \
  --handler bootstrap \
  --code S3Bucket=beobots-lambda-storage,S3Key=electricity/bootstrap.zip \
  --runtime provided.al2 \
  --role $ROLE_ARN \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active

```

#### Update the code changes

```bash
aws lambda update-function-code --function-name electricity_data_cron \
  --s3-bucket beobots-lambda-storage \
  --s3-key electricity/bootstrap.zip
```



### Invoke for testing

```bash
aws lambda invoke \
  --cli-binary-format raw-in-base64-out \
  --function-name electricity_data_cron
```

### Teardown

```bash
# nuke bucket contents (optional)
aws s3 rm s3://beobots-lambda-storage --recursive
# nuke bucket (optional)
aws s3api delete-bucket --bucket beobots-lambda-storage

# delete lambda
aws lambda delete-function --function-name electricity_data_cron

# policies and roles
aws iam detach-role-policy --role-name electricity_lambda_execution_role --policy-arn $POLICY_ARN
aws iam detach-role-policy --role-name electricity_lambda_execution_role --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

aws iam delete-policy --policy-arn $POLICY_ARN
aws iam delete-role --role-name electricity_lambda_execution_role

```



