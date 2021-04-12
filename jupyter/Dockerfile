FROM jupyter/datascience-notebook

# Settings / Configuration files
COPY config.py /home/jovyan/.jupyter/jupyter_notebook_config.py
# We set custom root for the Jupyter in ~/work
RUN mkdir -p /home/jovyan/work/projects
