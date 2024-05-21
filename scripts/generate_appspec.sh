#!/bin/bash

# Check if TASK_DEFINITION_ARN is set
if [ -z "$TASK_DEFINITION_ARN" ]; then
  echo "TASK_DEFINITION_ARN is not set"
  exit 1
fi

# Generate the appspec.yaml file
cat <<EOF > appspec.yaml
version: 0.0
Resources:
  - TargetService:
      Type: AWS::ECS::Service
      Properties:
        TaskDefinition: $TASK_DEFINITION_ARN
        LoadBalancerInfo:
          ContainerName: "ecs-service-container"
          ContainerPort: 80
