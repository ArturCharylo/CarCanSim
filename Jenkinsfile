pipeline {
    agent any 

    options {
        timestamps()
        disableConcurrentBuilds() 
    }

    stages {
        stage('Test & Lint (Rust)') {
            steps {
                sh 'docker build --target builder -t telemetry-builder .'
                
                sh 'docker run --rm telemetry-builder cargo test'
            }
        }

        stage('Build Environment') {
            steps {
                sh 'docker compose -p carcansim build'
            }
        }

        stage('Deploy Locally') {
            steps {
                sh 'docker compose -p carcansim up -d --build'
            }
        }
    }

    post {
        always {
            // Clean up dangling Docker images to free up WSL disk space
            sh 'docker image prune -f'
        }
        success {
            echo "Pipeline executed successfully! Telemetry app, Prometheus, and Grafana are up and running."
        }
        failure {
            echo "Pipeline failed. Check the Jenkins console output for troubleshooting."
        }
    }
}