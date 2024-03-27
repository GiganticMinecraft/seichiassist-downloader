import "@typespec/http";

using TypeSpec.Http;

@service({
  title: "seichiassist-downloader",
})
@server("http://localhost", "Development server")

namespace SeichiAssistDownloader {
  @route("/stable")
  @get
  @summary("最新の安定版SeichiAssistをダウンロードする")
  op getStable(): {
    @statusCode statusCode: 200,
    @header contentType: "application/java-archive",
    @body body: bytes,
  } | {
    @statusCode statusCode: 503,
    @header contentType: "application/json",
    @body body: {
      message: string,
    },
  };

  @route("/publish/stable")
  @post
  @summary("安定版ブランチに更新があったことを通知する")
  op publishStable(): {
    @statusCode statusCode: 200
  } | {
    @statusCode statusCode: 500
  };

  @route("/develop")
  @get
  @summary("最新の開発版SeichiAssistをダウンロードする")
  op getDevelopment(): {
    @statusCode statusCode: 200,
    @header contentType: "application/java-archive",
    @body body: bytes,
  } | {
    @statusCode statusCode: 503,
    @header contentType: "application/json",
    @body body: {
      message: string,
    },
  };

  @route("/publish/develop")
  @post
  @summary("開発版ブランチに更新があったことを通知する")
  op publishDevelopment(): {
    @statusCode statusCode: 200;
  } | {
    @statusCode statusCode: 500
  };
};
