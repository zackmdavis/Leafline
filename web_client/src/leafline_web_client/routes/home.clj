(ns leafline-web-client.routes.home
  (:require [leafline-web-client.layout :as layout]
            [compojure.core :refer [defroutes GET POST]]
            [ring.util.http-response :refer [ok]]
            [clojure.java.io :as io]
            [clojure.java.shell :refer [sh]]))

(defn home-page []
  (layout/render "home.html"))

(defn about-page []
  (layout/render "about.html"))

(defn correspondence-endpoint [request]
  (let [world (get-in request [:params :world])
        dictation ((sh "./Leafline"
                       "--lookahead" "4"
                       "--correspond" world) :out)]
    {:status 200
     :headers {"Content-Type" "application/json"}
     :body dictation}))

(defroutes home-routes
  (GET "/" [] (home-page))
  (GET "/about/" [] (about-page))
  (POST "/write/" [] correspondence-endpoint))
