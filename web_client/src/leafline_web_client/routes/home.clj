(ns leafline-web-client.routes.home
  (:require [leafline-web-client.layout :as layout]
            [compojure.core :refer [defroutes GET POST]]
            [ring.util.http-response :refer [ok]]
            [clojure.java.io :as io]
            [clojure.java.shell :refer [sh]]
            [taoensso.timbre :as timbre]))

(defn home-page []
  (layout/render "home.html"))

(defn about-page []
  (layout/render
   "about.html"
   {:readme (-> "project_readme.md" io/resource slurp)}))

(defn correspondence-endpoint [request]
  ;; TODO validate world, 400 if bad
  (let [world (str (get-in request [:params :world])
                   " b")  ; the AI plays blue (by convention, for now)
        mail-call ["./Leafline" "--lookahead" "5" "--correspond" world]]
    (timbre/info "got postcard about " world
                 "; invoking Leafline with" mail-call)
    (let [dictation (apply sh mail-call)]
      (if (zero? (dictation :exit))
        {:status 200
         ;; TODO RESEARCH: JSON helpers
         :headers {"Content-Type" "application/json"}
         :body (dictation :out)}
        {:status 500
         :headers {"Content-Type" "application/json"}
         :body {:error (dictation :err)}}))))

(defroutes home-routes
  (GET "/" [] (home-page))
  (GET "/about/" [] (about-page))
  (POST "/write/" [] correspondence-endpoint))
