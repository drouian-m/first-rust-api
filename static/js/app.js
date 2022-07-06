import {getTweets, appendTweets, postTweet, likeTweet} from "./tweet-service.js";

window.onload = async function () {
  const tweetDom = document.getElementById("tweets");

  const tweets = await getTweets();
  appendTweets(tweetDom, tweets)

  document.getElementById("tweetbtn").addEventListener("click", async function () {
    const tweet = await postTweet();
    appendTweets(tweetDom, [tweet])
  })
};
