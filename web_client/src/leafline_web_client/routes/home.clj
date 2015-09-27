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

(defn reasonable-request-validator [& {:keys [max-seconds max-depth]}]
  (fn [nature value]
    (condp = nature
      "seconds" (<= (Integer/parseInt value) max-seconds)
      "depth" (<= (Integer/parseInt value) max-depth))))

(defn json-response [response-map]
  (merge {:status 200
          :headers {"Content-Type" "application/json"}}
         response-map))

(def max-depth 7)
(def max-seconds 120)
(def reasonable-request? (reasonable-request-validator
                          :max-seconds 120 :max-depth 8))

(defn correspondence-endpoint [request]
  (let [world (str (get-in request [:params :world]))
        {:keys [nature value]} (get-in request [:params :bound])
        mail-call ["./leafline"
                   (str "--" nature) value
                   "--from" world
                   "--correspond"]]
    (timbre/info "got postcard about " world)
    (if (reasonable-request? nature value)
      (do
        (timbre/info "invoking Leafline with" mail-call)
        (let [dictation (apply sh mail-call)]
        (if (zero? (dictation :exit))
          (json-response {:body (dictation :out)})
          (json-response {:status 500
                          :body {:error (dictation :err)}}))))
      (json-response
       {:status 400
        :body {:error
               (format
                (str
                 "This deployment of Leafline is configured to think "
                 "for no more than %s seconds or to a depth of %s plies. "
                 "Please try a less-expensive request.")
                max-seconds max-depth)}}))))

(defroutes home-routes
  (GET "/" [] (home-page))
  (GET "/about/" [] (about-page))
  (POST "/write/" [] correspondence-endpoint))
