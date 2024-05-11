from setuptools import setup
from MosartDriver import __version__, __author__, __description__, __email__, __github__

setup(name='Mosart',
      version=__version__,
      description=__description__,
      author=__author__,
      author_email=__email__,
      url=__github__,
      packages=['MosartDriver'],
      classifiers=[
            'Development Status :: 5 - Production/Stable',
            'Operating System :: Microsoft :: Windows :: Windows 10',
            'Programming Language :: Python :: 3',
      ],
      install_requires=["pillow"]
     )
