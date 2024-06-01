import * as cdk from 'aws-cdk-lib';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import * as codedeploy from 'aws-cdk-lib/aws-codedeploy';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as codestarconnections from 'aws-cdk-lib/aws-codestarconnections';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import * as iam from 'aws-cdk-lib/aws-iam';
import { Construct } from 'constructs';
import { CodePipelineConfig, CodeSourceConfig, TaskDefConfig } from '../config/type';
import { RdsInstanceProps } from './rds-stack';

interface PipelineStackProps extends cdk.StackProps {
  config: CodePipelineConfig;
  taskDefConfig: TaskDefConfig;
  repository: ecr.Repository;
  rdsProps: RdsInstanceProps;
  taskDefinition: ecs.FargateTaskDefinition;
  ecsService: ecs.FargateService;
  albListener: elb.ApplicationListener;
  targetGroupBlue: elb.ApplicationTargetGroup;
  targetGroupGreen: elb.ApplicationTargetGroup;
}

export class CodePipelineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: PipelineStackProps) {
    super(scope, id, props);

    const {
      config,
      taskDefConfig,
      repository,
      rdsProps,
      taskDefinition,
      ecsService,
      albListener,
      targetGroupBlue,
      targetGroupGreen,
    } = props;

    const codeSourceOutput = new pipeline.Artifact();
    const codeBuildOutput = new pipeline.Artifact();

    const codeSourceStage = this.codeSourceStage({
      config: config.codeSource,
      output: codeSourceOutput,
    });

    const codeBuildStage = this.codeBuildStage({
      taskDefConfig,
      repository,
      rdsProps,
      taskDefinition,
      codeSourceOutput,
      output: codeBuildOutput,
    });

    const codeDeployStage = this.codeDeployStage({
      ecsService,
      albListener,
      targetGroupBlue,
      targetGroupGreen,
      codeBuildOutput,
    });

    new pipeline.Pipeline(this, 'Pipeline', {
      stages: [codeSourceStage, codeBuildStage, codeDeployStage],
    });
  }

  private codeSourceStage(props: CodeSourceProps): pipeline.StageProps {
    const { config, output } = props;

    const connection = new codestarconnections.CfnConnection(this, 'GitHubConnection', {
      connectionName: 'GitHubConnection',
      providerType: 'GitHub',
    });

    const action = new pipelineactions.CodeStarConnectionsSourceAction({
      actionName: 'CodeStarSource',
      owner: config.owner,
      repo: config.repo,
      branch: config.branch,
      connectionArn: connection.attrConnectionArn,
      output: output,
    });
    return {
      stageName: 'Source',
      actions: [action],
    };
  }

  private codeBuildStage(props: CodeBuildProps): pipeline.StageProps {
    const { taskDefConfig, repository, rdsProps, taskDefinition, codeSourceOutput, output } = props;

    console.log("\n\n\ntaskdef\n\n\n");
    console.log(taskDefinition.toString());

    const codeBuildProject = new codebuild.PipelineProject(this, 'CodeBuildProject', {
      cache: codebuild.Cache.local(codebuild.LocalCacheMode.DOCKER_LAYER),
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.SMALL,
        privileged: true,
        environmentVariables: {
          REPOSITORY_URI: { value: repository.repositoryUri },
          IMAGE_TAG: { value: taskDefConfig.imageTag },
          IMAGE_REPO_NAME: { value: repository.repositoryName },

          TASK_DEF_ARN: { value: taskDefinition.taskDefinitionArn },
          TASK_DEF_ROLE_ARN: { value: taskDefinition.taskRole.roleArn },
          TASK_DEF_EXEC_ROLE_ARN: { value: taskDefinition.executionRole?.roleArn },

          TASK_DEF_FAMILY: { value: taskDefinition.family },

          TASK_DEF_MEM_LIMIT_MIB: { value: "2048" },
          TASK_DEF_CPU: { value: "1024" },
          TASK_DEF_CONTAINER_PORT: { value: 9999 },

          TASK_DEF_HC_COMMAND: { value: taskDefConfig.healthCheck.command },
          TASK_DEF_HC_INTERVAL: { value: taskDefConfig.healthCheck.intervalSec },
          TASK_DEF_HC_RETRIES: { value: taskDefConfig.healthCheck.unhealthyThresholdCount },
          TASK_DEF_HC_TIMEOUT: { value: taskDefConfig.healthCheck.timeoutSec },
          TASK_DEF_HC_START_PERIOD: { value: taskDefConfig.healthCheck.startPeriodSec },

          TASK_DEF_DB_USERNAME: { value: rdsProps.credentials.username },
          TASK_DEF_DB_HOST: { value: rdsProps.address },
          TASK_DEF_DB_PORT: { value: rdsProps.port },
          TASK_DEF_DB_NAME: { value: rdsProps.databaseName },
          TASK_DEF_DB_PASSWORD: { value: `${rdsProps.credentials.secret?.secretArn}:password` },
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

    const action = new pipelineactions.CodeBuildAction({
      actionName: 'Build',
      project: codeBuildProject,
      input: codeSourceOutput,
      outputs: [output],
    });

    return {
      stageName: 'Build',
      actions: [action],
    };
  }

  private codeDeployStage(props: CodeDeployProps): pipeline.StageProps {
    const { ecsService, albListener, targetGroupBlue, targetGroupGreen, codeBuildOutput } = props;

    const application = new codedeploy.EcsApplication(this, 'EcsApplication');

    const deploymentGroup = new codedeploy.EcsDeploymentGroup(this, 'EcsDeploymentGroup', {
      application,
      service: ecsService,
      deploymentConfig: codedeploy.EcsDeploymentConfig.ALL_AT_ONCE,
      blueGreenDeploymentConfig: {
        listener: albListener,
        blueTargetGroup: targetGroupBlue,
        greenTargetGroup: targetGroupGreen,
      },
      autoRollback: {
        failedDeployment: true,
      },
    });

    const action = new pipelineactions.CodeDeployEcsDeployAction({
      actionName: 'EcsDeploy',
      appSpecTemplateInput: codeBuildOutput,
      taskDefinitionTemplateInput: codeBuildOutput,
      deploymentGroup: deploymentGroup,
    });

    return {
      stageName: 'EcsDeploy',
      actions: [action],
    };
  }
}

interface CodeSourceProps {
  config: CodeSourceConfig;
  output: pipeline.Artifact;
}

interface CodeBuildProps {
  taskDefConfig: TaskDefConfig;
  repository: ecr.Repository;
  rdsProps: RdsInstanceProps;
  taskDefinition: ecs.FargateTaskDefinition;
  codeSourceOutput: pipeline.Artifact;
  output: pipeline.Artifact;
}

interface CodeDeployProps {
  ecsService: ecs.FargateService;
  albListener: elb.ApplicationListener;
  targetGroupBlue: elb.ApplicationTargetGroup;
  targetGroupGreen: elb.ApplicationTargetGroup;
  codeBuildOutput: pipeline.Artifact;
}
