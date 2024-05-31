import * as cdk from 'aws-cdk-lib';
import * as codestarconnections from 'aws-cdk-lib/aws-codestarconnections';
import { Construct } from 'constructs';

export class CodeStarStack extends cdk.Stack {
  public readonly connectionArn: string;

  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const connection = new codestarconnections.CfnConnection(this, 'GitHubConnection', {
      connectionName: 'GitHubConnection',
      providerType: 'GitHub',
    });

    this.connectionArn = connection.attrConnectionArn;

    new cdk.CfnOutput(this, 'GitHubConnectionArn', {
      value: this.connectionArn,
    });
  }
}
