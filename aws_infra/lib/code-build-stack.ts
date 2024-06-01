import * as cdk from 'aws-cdk-lib';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import { Construct } from 'constructs';

interface CodeBuildProps extends cdk.StackProps {
  sourceOutput: pipeline.Artifact;
  repository: ecr.Repository;
  taskDefinitionArn: string;
}

export class CodeBuildStack extends cdk.Stack {
  public readonly action: pipelineactions.CodeBuildAction;
  public readonly output: pipeline.Artifact;

  constructor(scope: Construct, id: string, props: CodeBuildProps) {
    super(scope, id, props);

    const { sourceOutput, repository, taskDefinitionArn } = props;

    const project = new codebuild.Project(this, 'CodeBuildProject', {
      cache: codebuild.Cache.local(codebuild.LocalCacheMode.DOCKER_LAYER),
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.SMALL,
        privileged: true,
        environmentVariables: {
          REPOSITORY_URI: {
            value: repository.repositoryUri,
          },
          IMAGE_REPO_NAME: {
            value: repository.repositoryName,
          },
          TASK_DEFINITION_ARN: {
            value: taskDefinitionArn,
          },
        },
      },
      buildSpec: codebuild.BuildSpec.fromSourceFilename('buildspec.yaml'),
    });

    this.output = new pipeline.Artifact();

    this.action = new pipelineactions.CodeBuildAction({
      actionName: 'Build',
      project,
      input: sourceOutput,
      outputs: [this.output],
    });
  }
}
