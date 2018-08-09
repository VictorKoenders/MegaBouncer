import * as React from "react";

export interface Props { }
export interface State {
    state: server.State | null,
}

export class Root extends React.Component<Props, State> {
    interval: number;
    constructor(props: Props, context?: any) {
        super(props, context);
        this.state = {
            state: null,
        };
    }
    componentWillMount() {
        this.interval = setInterval(this.fetch.bind(this), 1000);
        this.fetch();
    }
    fetch() {
        fetch("/api/state")
            .then(r => r.json())
            .then((r: server.State) => {
                this.setState({
                    state: r
                });
            });
    }
    render_running_build(build: server.RunningBuild, index: number): JSX.Element {
        return <p key={index}>
            <b>{build.build}</b><br />
            <pre>{build.stdout}</pre>
            <pre>{build.stderr}</pre>
        </p>;
    }
    render_finished_build(build: server.FinishedBuild, index: number): JSX.Element {
        return <p key={index}>
            <b>{build.build}</b><br />
            <pre>{build.stdout}</pre>
            <pre>{build.stderr}</pre>
        </p>;
    }
    render_process(process: server.RunningProcess, index: number) {
        return <p key={index}>
            <b>{process.directory}</b><br />
            <pre>{process.stdout}</pre>
            <pre>{process.stderr}</pre>
        </p>;
    }
    render_project(project: server.Project, index: number) {
        return <p key={index}><b>{project.name}</b> {project.builds.map(this.render_build.bind(this, project))}</p>;
    }
    render_build(project: server.Project, build: server.Build, index: number) {
        return <button key={index} onClick={this.start_build.bind(this, project, build)}>{build.name}</button>;
    }
    start_build(project: server.Project, build: server.Build, ev: React.MouseEvent<HTMLButtonElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        fetch("/api/build/start/" + project.name + "/" + build.name).then(r => r.text()).then(t => {
            if(t != "Ok") {
                alert("Could not start build\n" + t);
            }
        });

        return false;
    }
    render() {
        if (!this.state.state) return <></>;
        return <>
            <div style={{ display: 'flex', flexDirection: 'row' }}>
                <div style={{width: '50%'}}>
                    <h2>Processes:</h2>
                    {this.state.state.running_processes.map(this.render_process.bind(this))}
                </div>
                <div style={{width: '50%'}}>
                    {this.state.state.projects.map(this.render_project.bind(this))}
                </div>
            </div>
            <div style={{ display: 'flex', flexDirection: 'row' }}>
                <div style={{width: '50%'}}>
                    <h2>Running:</h2>
                    {this.state.state.running_builds.map(this.render_running_build.bind(this))}
                </div>
                <div style={{width: '50%'}}>
                    <h2>Finished:</h2>
                    {this.state.state.finished_builds.map(this.render_finished_build.bind(this))}
                </div>
            </div>
        </>;
    }
}