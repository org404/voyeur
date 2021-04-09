FROM jupyter/minimal-notebook

COPY requirements.txt .
RUN pip3 install --no-cache-dir -U pip wheel setuptools \
 && pip3 install --no-cache-dir -r requirements.txt

# Settings / Configuration files
COPY config.py /home/jovyan/.jupyter/jupyter_notebook_config.py
# COPY .config/passwords.json /home/jovyan/.jupyter/jupyter_notebook_config.json
# We set custom root for the Jupyter in ~/work
RUN mkdir -p /home/jovyan/work/projects
