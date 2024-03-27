# seichiassist-downloader

[SeichiAssist](https://github.com/GiganticMinecraft/SeichiAssist/)の jar ファイルをダウンロードするためのソフトウェア

# 作成した経緯

前提として、整地鯖を 1.18 にアップデートすると同時に整地鯖自体を k8s([seichi_infra](https://github.com/GiganticMinecraft/seichi_infra)) 上に乗せることになった。
このとき、SeichiAssist のリリース(jar ファイルのバージョン)には以下 3 つの種類が必要だった。

- 本番環境用の安定版(master ブランチ)
- 検証環境用の開発版(develop ブランチ)
- プルリクエストごとの環境用の開発版(動的に指定される(PR にタグが付けられた)ブランチ)

これらのリリースを別々に取得できた上で、本番環境用の安定版は開発者がこの状態でリリースをするという意思表明を行う操作(develop ブランチから master ブランチへのマージ操作(正確には、GitHub Actions による操作))が行われたうえで安定版をリリースするという必要があった。これを解決するために、seichiassist-downloader を作成した。

# 仕組み

![seichiassist-downloaderの俯瞰図](./docs/overview.drawio.svg)

# 環境変数

| 環境変数名     | 例      |
| -------------- | ------- |
| HTTP_PORT      | 80      |
| STABLE_BRANCH  | master  |
| DEVELOP_BRANCH | develop |
