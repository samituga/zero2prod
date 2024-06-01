import * as cdk from 'aws-cdk-lib';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as codestarconnections from 'aws-cdk-lib/aws-codestarconnections';
import { Construct } from 'constructs';
import { CodeSourceConfig } from '../config/type';

interface CodeStarProps extends cdk.StackProps {
  config: CodeSourceConfig;
}

export class CodeSourceStack extends cdk.Stack {
  public readonly action: pipelineactions.CodeStarConnectionsSourceAction;
  public readonly output: pipeline.Artifact;

  constructor(scope: Construct, id: string, props: CodeStarProps) {
    super(scope, id, props);

    const { config } = props;

    const connection = new codestarconnections.CfnConnection(this, 'GitHubConnection', {
      connectionName: 'GitHubConnection',
      providerType: 'GitHub',
    });

    this.output = new pipeline.Artifact();

    this.action = new pipelineactions.CodeStarConnectionsSourceAction({
      actionName: 'CodeStarSource',
      owner: config.owner,
      repo: config.repo,
      branch: config.branch,
      connectionArn: connection.attrConnectionArn,
      output: this.output,
    });

    new cdk.CfnOutput(this, 'GitHubConnectionArn', {
      value: connection.attrConnectionArn,
    });
  }
}
