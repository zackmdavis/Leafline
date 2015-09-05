(ns leafline-web-client.routes.home
  (:require [leafline-web-client.layout :as layout]
            [compojure.core :refer [defroutes GET POST]]
            [ring.util.http-response :refer [ok]]
            [clojure.java.io :as io]))

(defn home-page []
  (layout/render
    "home.html"))

(defn about-page []
  (layout/render "about.html"))

(defn correspondence-endpoint []
  "{\"testing\": true}")

(defroutes home-routes
  (GET "/" [] (home-page))
  (GET "/about/" [] (about-page))
  (POST "/write/" [] (correspondence-endpoint))
)
