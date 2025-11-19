/*
 * (c) Copyright, Real-Time Innovations, 2024.  All rights reserved.
 * RTI grants Licensee a license to use, modify, compile, and create derivative
 * works of the software solely for use with RTI Connext DDS. Licensee may
 * redistribute copies of the software provided that all such copies are subject
 * to this license. The software is provided "as is", with no warranty of any
 * type, including any warranty for fitness for any purpose. RTI is under no
 * obligation to maintain or support the software. RTI shall not be liable for
 * any incidental or consequential damages arising out of the use or inability
 * to use the software.
 */

pipeline {
    agent {
        dockerfile {
            dir 'resources/docker'
            reuseNode true
            label 'docker'
        }
    }

    options {
        disableConcurrentBuilds()
        // Set a timeout for the entire pipeline
        timeout(time: 30, unit: 'MINUTES')
    }

    triggers {
        // Run daily at 2:00 AM
        cron('0 2 * * *')
        // Run when new commits are pushed to the repository
        pollSCM('H/30 * * * *')
    }

    stages {
        stage('Build doc') {
            steps {
                script {
                    downloadAndExtract(
                            installDirectory: '.',
                            flavour: 'connectorlibs'
                        )

                    def cargoDocFlags = [
                        '--no-deps',
                        '--all-features',
                        '--document-private-items',
                        '--examples',
                        '--bins',
                        '--lib',
                    ].join(' ')
                    sh "cargo doc ${cargoDocFlags}"
                }
            }

            post {
                success {
                    publishHTML(
                        [
                            allowMissing: false,
                            alwaysLinkToLastBuild: true,
                            keepAll: false,
                            reportDir: 'target/doc/',
                            reportFiles: 'rtiddsconnector/index.html',
                            reportName: 'Connector Documentation',
                            reportTitles: 'Connector Documentation'
                        ]
                    )
                }
            }
        }
    }

    post {
        cleanup {
            cleanWs()
        }
    }
}
