'use client';

import React from 'react';

function ArticleSelect({ text, setText }: { text: string, setText: (value: string) => void }) {
  const [abort, setAbort] = React.useState(new AbortController());
  const [results, setResults] = React.useState([] as string[]);
  const [resultPending, setResultPending] = React.useState(0);

  React.useEffect(() => {
    abort.abort('subsequent request');

    const signal = new AbortController();
    setAbort(signal);

    setResultPending(p => p + 1);
    fetch('http://159.89.230.173:3000/search', {
      method: 'POST',
      body: `%${text}%`,
      signal: signal.signal,
    })
      .then(res => res.text())
      .then(res => res.split('\n'))
      .then(results => {
        setResults(results);
      })
      .catch(e => {
        console.warn('Request failed:', e);
      })
      .finally(() => {
        setResultPending(p => p - 1);
      });
  }, [text]);

  return (
    <div className={`flex bg-gray-700 rounded-xl px-3 ring-1 ring-gray-800 shadow-lg shadow-sky-900/30 relative even:after:content-['To'] odd:after:content-['From'] after:absolute after:-top-1 after:left-2 after:bg-gray-600 after:font-semibold after:shadow-md after:ring-1 after:ring-gray-500 after:w-fit after:px-2 after:h-7 after:leading-7 after:rounded-full after:-translate-x-6 after:-translate-y-1/2 before:right-2 before:h-4 before:w-8 before:text-left before:top-2.5 before:absolute before:animate-loading before:text-3xl before:leading-4 before:text-white ${resultPending ? 'before:visible' : 'before:invisible'}`}>
      <input type="text" value={text} onChange={e => setText(e.currentTarget.value)} className="peer rounded bg-transparent text-white focus:outline-none pl-2 py-3 text-lg font-medium" />
      <ul className="hidden hover:block peer-focus:block absolute bg-neutral-900 rounded-lg p-2 space-y-1 text-left top-full left-0 max-h-48 min-w-72 overflow-scroll">
        {results.map(result => (
          <li className="even:bg-neutral-800 rounded-md px-4" key={result}>
            <button className="text-left" onClick={() => setText(result)}>{result}</button>
          </li>
        ))}

        {!results.length && (
          <li className="opacity-75">Enter a search query</li>
        )}
      </ul>
    </div>
  );
}

export default function Home() {
  const [from, setFrom] = React.useState('');
  const [to, setTo] = React.useState('');

  const [path, setPath] = React.useState([] as string[]);
  const [error, setError] = React.useState('');
  const [resultPending, setResultPending] = React.useState(null as null | 'bfs' | 'dfs');

  const search = (alg: 'bfs' | 'dfs') => {
    setResultPending(alg);

    fetch('http://159.89.230.173:3000/' + alg, {
      method: 'POST',
      body: `${from}\n${to}`
    })
      .then(async res => {
        if (res.status === 404) {
          setError('Start or end article not found');
          setPath([]);
        } else if (res.status === 418) {
          setError('No path exists between articles');
          setPath([]);
        } else if (res.ok) {
          const path = await res.text();
          setPath(path.split('\n'));
          setError('');
        } else {
          setError('An unknown error occurred');
          setPath([]);
        }
      })
      .catch(err => {
        console.warn('Request failed:', err);
      })
      .finally(() => {
        setResultPending(null);
      });
  };

  return (
    <div className="max-w-lg mx-auto text-center my-10">
      <h1 className="text-5xl font-semibold">Wiki<span className="text-gray-400">Links</span></h1>
      <h3 className="text-blue-400 my-2">COP3530 Final Project</h3>

      <div className="grid grid-cols-2 gap-x-6 gap-y-4 mt-12">
        <ArticleSelect text={from} setText={setFrom} />
        <ArticleSelect text={to} setText={setTo} />

        <div className="col-span-2 flex flex-col items-center gap-y-2">
          <button onClick={() => search('bfs')} disabled={!!resultPending && resultPending !== 'bfs'} className="disabled:bg-gray-900 bg-gray-800 hover:bg-gray-700 py-1 shadow-lg w-52 rounded-lg transition-colors">Breadth-First Search</button>
          <button onClick={() => search('dfs')} disabled={!!resultPending && resultPending !== 'dfs'} className="disabled:bg-gray-900 bg-gray-800 hover:bg-gray-700 py-1 shadow-lg w-52 rounded-lg transition-colors">Depth-First Search</button>

          <div className="mt-4 space-y-2">
            <h2 className="text-xl font-bold">Path ({path.length})</h2>
            {(error || !path.length) && (
              <p>{error ? error : 'Select articles then start a search'}</p>
            )}

            {!!path.length && (
              <ul>
                {path.map(article => (
                  <li key={article}>
                    <a className="hover:underline" target="_blank" href={`https://en.wikipedia.org/wiki/${article.replace(' ', '_')}`}>{article}</a>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
