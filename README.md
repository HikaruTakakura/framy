# Framy

Framy is a command-line tool that adds frames to images.

## Installation

### via Homebrew

```sh
brew tap HikaruTakakura/framy
brew install framy
```

## Usage

```sh
# add default frame
framy images/horses.jpg --outdir images
```

```sh
# add black frame
framy images/horses.jpg --outdir images --color 000000 --suffix _black
```

| path                    | image                                               |
| ----------------------- | --------------------------------------------------- |
| images/horses.jpg       | <img src="./images/horses.jpg" width="320px">       |
| images/horses_framy.jpg | <img src="./images/horses_framy.jpg" width="320px"> |
| images/horses_black.jpg | <img src="./images/horses_black.jpg" width="320px"> |

```sh
#  add frames only to images that have a 'favorite' tag
DIR=./images && mdfind -onlyin $DIR "kMDItemUserTags == favorite" | framy --outdir "${DIR}/_framy"

tree ./images
./images
├── _framy
│   ├── dog_framy.jpg
│   └── cat_framy.jpg
├── dog.jpg
├── ogre.jpg
└── cat.jpg
```

## License

This project is released under the MIT license. For details, please refer to the `LICENSE` file.
