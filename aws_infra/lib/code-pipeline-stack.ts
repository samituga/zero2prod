import * as cdk from 'aws-cdk-lib';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import { Construct } from 'constructs';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as iam from 'aws-cdk-lib/aws-iam';

interface PipelineStackProps extends cdk.StackProps {
  codeSourceAction: pipelineactions.CodeStarConnectionsSourceAction;
  codeSourceOutput: pipeline.Artifact;
  repository: ecr.Repository;
  taskDefinitionArn: string;
}

export class CodePipelineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: PipelineStackProps) {
    super(scope, id, props);

    const { codeSourceAction, codeSourceOutput, repository, taskDefinitionArn } = props;

    const codeBuildProject = new codebuild.PipelineProject(this, 'CodeBuildProject', {
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
      buildSpec: codebuild.BuildSpec.fromSourceFilename('aws_infra/buildspec.yaml'),
    });

    codeBuildProject.addToRolePolicy(
      new iam.PolicyStatement({
        actions: [
          'ecr:GetDownloadUrlForLayer',
          'ecr:BatchGetImage',
          'ecr:BatchCheckLayerAvailability',
          'ecr:GetAuthorizationToken',
        ],
        resources: ['*'],
      }),
    );

    const codeBuildOutput = new pipeline.Artifact();

    const codeBuildAction = new pipelineactions.CodeBuildAction({
      actionName: 'Build',
      project: codeBuildProject,
      input: codeSourceOutput,
      outputs: [codeBuildOutput],
    });

    const codePipeline = new pipeline.Pipeline(this, 'Pipeline', {
      stages: [
        {
          stageName: 'Source',
          actions: [codeSourceAction],
        },
        {
          stageName: 'Build',
          actions: [codeBuildAction],
        },
      ],
    });

    new cdk.CfnOutput(this, 'PipelineName', {
      value: codePipeline.pipelineName,
    });
  }
}
