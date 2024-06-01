import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import { Construct } from 'constructs';
import { RdsConfig } from '../config/type';

interface RdsStackProps extends cdk.StackProps {
  config: RdsConfig;
  vpc: ec2.Vpc;
  sg: ec2.SecurityGroup;
}

export class RdsStack extends cdk.Stack {
  public readonly dbProps: RdsInstanceProps;

  constructor(scope: Construct, id: string, props: RdsStackProps) {
    super(scope, id, props);

    const { config, vpc, sg } = props;

    const subnetGroup = new rds.SubnetGroup(this, 'RdsSubnetGroup', {
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_ISOLATED },
      description: 'Subnet group for the RDS instance',
    });

    const rdsSecret = new secretsmanager.Secret(this, 'RdsCredentials', {
      secretName: config.databaseName + 'RdsCredentials',
      description: config.databaseName + 'RDS Database Crendetials',
      generateSecretString: {
        excludeCharacters: '"@/\\ \'',
        generateStringKey: 'password',
        passwordLength: 30,
        secretStringTemplate: JSON.stringify({ username: config.username }),
      },
    });

    const rdsCredentials = rds.Credentials.fromSecret(rdsSecret, config.username);

    const dbInstance = new rds.DatabaseInstance(this, 'RdsInstance', {
      databaseName: config.databaseName,
      instanceIdentifier: config.databaseName,
      credentials: rdsCredentials,
      allocatedStorage: config.allocatedStorage,
      maxAllocatedStorage: config.maxAllocatedStorage,
      instanceType: new cdk.aws_ec2.InstanceType(config.instanceType),
      port: config.port,
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_16_2,
      }),
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      monitoringInterval: cdk.Duration.seconds(60),
      enablePerformanceInsights: true,
      multiAz: true,
      publiclyAccessible: false,

      vpc,
      securityGroups: [sg],
      subnetGroup,
    });

    this.dbProps = {
      address: dbInstance.dbInstanceEndpointAddress,
      databaseName: config.databaseName,
      port: config.port,
      credentials: rdsCredentials,
    };
  }
}

export interface RdsInstanceProps {
  address: string;
  databaseName: string;
  port: number;
  credentials: rds.Credentials;
}
