import React, { useState } from 'react';
import { FaGithubSquare, FaHeart, FaRegHeart, FaStar } from 'react-icons/fa';
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { FiRefreshCw } from 'react-icons/fi';
import { FaSquareGitlab } from 'react-icons/fa6';
import { MdDelete } from 'react-icons/md';

interface Ok<T> {
  Ok: T,
}

interface Error {
  Err: string,
}

type Result<T> = Ok<T> | Error

enum GitProvider {
  GitHub = "GitHub",
  GitLab = "GitLab",
}

interface Repo {
  git_type: GitProvider,
  owner: string,
  name: string,
}

interface Setting {
  id: string,
  order: number,
  favourite: boolean,
  repo: Repo,
}

interface Fetched {
  setting: Setting,
  stars: number,
}

//Init data 
async function init_data(): Promise<Fetched[]> {
  const read: Result<Fetched>[] = await invoke("read");
  return read.filter((data) => data && "Ok" in data).map((data) => { return data.Ok }).sort((a, b) => a.setting.favourite ? -1 : a.setting.order - b.setting.order)
}

const data: Fetched[] = await init_data()

export default function RepoList() {

  const [repos, setRepos] = useState<Fetched[]>(data);
  const [repoSelectedProvider, setRepoSelectedProvider] = useState(GitProvider.GitHub);
  const [repoName, setRepoName] = useState('');
  const [repoOwner, setRepoOwner] = useState('');
  const [errorIsOpen, setErrorIsOpen] = useState(false);

  const refresh = async function () {
    setRepos(await init_data())
  }

  const addRepo = async function () {
    if (repoName && repoOwner) {
      const repo: Repo = {
        git_type: repoSelectedProvider,
        owner: repoOwner,
        name: repoName,
      };
      const id: string = await invoke("uuid");
      const setting: Setting = {
        id,
        order: repos.length,
        favourite: false,
        repo
      }
      try {
        const stars: number = await invoke("create", { setting })
        if (typeof stars === "number") {
          const fetched: Fetched = {
            setting,
            stars
          }
          setRepos([...repos, fetched]);
          setRepoName('');
          setRepoOwner('');
        }
      } catch (error) {
        console.log(error)
        setErrorIsOpen(true)
        setTimeout(() => setErrorIsOpen(false), 5000)
      }
    }
  };

  const getLogo = (provider: GitProvider) => {
    switch (provider) {
      case GitProvider.GitHub:
        return <div><FaGithubSquare className="size-8" /></div>;
      case GitProvider.GitLab:
        return <div><FaSquareGitlab className="size-8" /></div>;
      default:
        return null;
    }
  };

  const getStarsIcon = (stars: number) => {
    return (
      <div className="flex items-center justify-start">
        <div className='prose mx-2'>{stars}</div>
        <div><FaStar className="size-5 rounded-box" /></div>
      </div>
    );
  };

  const getFavouriteButton = (which: Fetched) => {
    return (
      <button className="btn btn-lg btn-circle btn-ghost"
        onClick={async () => { await modFavourite(which.setting) }}>
        {which.setting.favourite ? <FaHeart /> : <FaRegHeart />}
      </button>
    );
  };

  const getDeleteButton = (which: Fetched) => {
    return (
      <button className="btn btn-lg btn-circle btn-ghost"
        onClick={async () => { await modDelete(which.setting) }}>
        <MdDelete />
      </button>
    );
  };

  const modDelete = async function (which: Setting) {
    if (which.favourite)
      await modFavourite(which)
    await invoke("delete", { "setting": which })
    const new_repos = [...repos].filter(item => item.setting.id !== which.id)
    setRepos([...new_repos])
  }

  const modFavourite = async function (which: Setting) {
    which.favourite = !which.favourite
    const new_repos = [...repos]
    for (const f of new_repos) {
      if (f.setting.id === which.id) {
        f.setting.favourite = which.favourite
        await invoke("update", { "setting": which })
      } else if (f.setting.favourite === true && which.favourite === true) {
        f.setting.favourite = false
      }
    }
    setRepos([...new_repos])
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-xs md:max-w-5xl md:mx-auto">
        <fieldset className="fieldset bg-base-200 border-base-300 rounded-box border p-4 ">
          <legend className="fieldset-legend">Track new repo</legend>
          <div className="join join-vertical md:join-horizontal">
            <select className="md:flex-2 select select-md lg:select-lg join-item"
              name='providerSelector'
              value={repoSelectedProvider}
              onChange={(e) => setRepoSelectedProvider(GitProvider[e.target.value as keyof typeof GitProvider])}
            >
              {Object.values(GitProvider).map((provider, i) => <option value={provider} key={i}>{provider} </option>)}
            </select>
            <input type="text" className="md:flex-3 input input-md lg:input-lg join-item"
              value={repoOwner}
              name='ownerSelector'
              onChange={(e) => setRepoOwner(e.target.value)}
              placeholder="owner/org" />
            <input type="text" className="md:flex-3 input input-md lg:input-lg join-item"
              value={repoName}
              name='nameSelector'
              onChange={(e) => setRepoName(e.target.value)}
              placeholder="name" />
            <button className="md:flex-1 btn btn-primary btn-md lg:btn-lg join-item"
              onClick={addRepo}>
              Track
            </button>
          </div>
        </fieldset>
      </div>
      <div className='max-w-xs md:max-w-5xl md:mx-auto my-8'>
        <ul className="list bg-base-200 border-base-300 rounded-box shadow-md">
          {repos.map((repo, index) => (
            <li className="list-row items-center" key={index}>
              {getLogo(repo.setting.repo.git_type)}
              <div>
                <div className="prose">{repo.setting.repo.owner}/{repo.setting.repo.name}</div>
              </div>
              {getStarsIcon(repo.stars)}
              <div className='flex justify-end'>
                {getFavouriteButton(repo)}
                {getDeleteButton(repo)}
              </div>
            </li>
          ))}
        </ul>
      </div>
      <div className="toast">
        <button className="btn btn-secondary btn-md lg:btn-lg"
          onClick={refresh}>
          <FiRefreshCw />
          Refresh
        </button>
      </div>
      <div className="toast toast-top toast-end" hidden={!errorIsOpen}>
        <div className="alert alert-error">
          <span>Repo not available</span>
        </div>
      </div>
    </div>
  );
}