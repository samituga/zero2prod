import * as cdk from 'aws-cdk-lib';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import { Construct } from 'constructs';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as codestarconnections from 'aws-cdk-lib/aws-codestarconnections';
import { CodePipelineConfig, CodeSourceConfig } from '../config/type';

interface PipelineStackProps extends cdk.StackProps {
  config: CodePipelineConfig;
  repository: ecr.Repository;
  taskDefinitionArn: string;
}

export class CodePipelineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: PipelineStackProps) {
    super(scope, id, props);

    const { config, repository, taskDefinitionArn } = props;

    const codeSourceOutput = new pipeline.Artifact();
    const codeBuildOutput = new pipeline.Artifact();

    const codeSourceAction = this.codeSourceAction({
      config: config.codeSource,
      output: codeSourceOutput,
    });

    const codeBuildAction = this.codeBuildAction({
      repository,
      taskDefinitionArn,
      codeSourceOutput,
      output: codeBuildOutput,
    });

    new pipeline.Pipeline(this, 'Pipeline', {
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
  }

  private codeSourceAction(props: CodeSourceProps) {
    const { config, output } = props;

    const connection = new codestarconnections.CfnConnection(this, 'GitHubConnection', {
      connectionName: 'GitHubConnection',
      providerType: 'GitHub',
    });

    return new pipelineactions.CodeStarConnectionsSourceAction({
      actionName: 'CodeStarSource',
      owner: config.owner,
      repo: config.repo,
      branch: config.branch,
      connectionArn: connection.attrConnectionArn,
      output: output,
    });
  }

  private codeBuildAction(props: CodeBuildProps) {
    const { repository, taskDefinitionArn, codeSourceOutput, output } = props;

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
          'ecr:GetRepositoryPolicy',
          'ecr:DescribeRepositories',
          'ecr:ListImages',
          'ecr:DescribeImages',
          'ecr:ListTagsForResource',
          'ecr:DescribeImageScanFindings',
          'ecr:InitiateLayerUpload',
          'ecr:UploadLayerPart',
          'ecr:CompleteLayerUpload',
          'ecr:PutImage',
        ],
        resources: ['*'],
      }),
    );

    return new pipelineactions.CodeBuildAction({
      actionName: 'Build',
      project: codeBuildProject,
      input: codeSourceOutput,
      outputs: [output],
    });
  }
}

interface CodeSourceProps {
  config: CodeSourceConfig;
  output: pipeline.Artifact;
}

interface CodeBuildProps {
  repository: ecr.Repository;
  taskDefinitionArn: string;
  codeSourceOutput: pipeline.Artifact;
  output: pipeline.Artifact;
}
